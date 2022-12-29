use anyhow::Result;
use leakpolicy::MatchContext;
use std::collections::VecDeque;

mod parse;

use crate::{
    evaluator::{self, MatcherMetadata, MatcherState},
    perf::PerformanceMonitor,
    pipe::PipeReader,
    policy::{ContentType, Policy, PolicyAction},
    Match,
};

use super::{ParseResponse, Parser, ParserConfiguration};

/// returns (key, value) matcher states
fn prepare_match_state<'a>(
    policy: &'a Policy,
    configuration: &ParserConfiguration<'a>,
) -> (MatcherState<'a>, MatcherState<'a>) {
    let mut key_match_state = MatcherState::default();
    let mut value_match_state = MatcherState::default();

    for (category_name, action) in configuration.categories {
        if !action.search.match_specific(configuration.active_context)
            || !action
                .category_config
                .search
                .match_specific(configuration.active_context)
        {
            continue;
        }

        if !action.category_config.content_types.is_empty()
            && !action
                .category_config
                .content_types
                .contains(&ContentType::Json)
        {
            continue;
        }

        let mut do_key = true;
        let mut do_value = true;
        if !action.category_config.contexts.is_empty() {
            if !action
                .category_config
                .contexts
                .contains(&MatchContext::Keys)
            {
                do_key = false;
            }
            if !action
                .category_config
                .contexts
                .contains(&MatchContext::Values)
            {
                do_value = false;
            }
        }
        if !do_key && !do_value {
            continue;
        }

        if matches!(
            action.category_config.action.unwrap_or_default(),
            PolicyAction::Ignore
        ) {
            continue;
        }

        let metadata = MatcherMetadata {
            policy_path: action.matcher_path.clone(),
            category_name: category_name.to_string(),
            action: action.category_config.action.unwrap_or_default(),
            local_report_style: action.report_style,
            correlation: None,
        };

        if do_key {
            evaluator::prepare_matches(
                policy,
                category_name,
                &mut key_match_state,
                &metadata,
                &action.category_config.ignore,
                true,
            );
        }
        if do_value {
            evaluator::prepare_matches(
                policy,
                category_name,
                &mut value_match_state,
                &metadata,
                &action.category_config.ignore,
                true,
            );
        }
    }

    (key_match_state, value_match_state)
}

#[derive(Debug)]
struct SegmentMap {
    /// original starting index
    original: u64,
    /// start and end index in the current buffer
    buffered: (u64, u64),
}

/// wrapper around `MatcherState` to aid in batched matching
struct JsonMatcher<'a> {
    matcher: MatcherState<'a>,
    performance: &'a PerformanceMonitor,
    /// buffer of segments for the next batch of matching
    str_buf: String,
    /// index mappings corresponding to the current `str_buf`
    idx_map: VecDeque<SegmentMap>,
    /// buffer of matches with uncorrected indexes
    match_buf: Vec<Match>,
}

impl<'a> JsonMatcher<'a> {
    /// 10kb buffer limit
    const BATCH_SIZE_LIMIT: usize = 10_000;

    fn new(matcher: MatcherState<'a>, performance: &'a PerformanceMonitor) -> Self {
        Self {
            matcher,
            performance,
            str_buf: Default::default(),
            idx_map: Default::default(),
            match_buf: Default::default(),
        }
    }

    /// add some data and its index into the batch and perform matching if buf is larger than `Self::BATCH_SIZE_LIMIT`
    fn process(
        &mut self,
        matches: &mut Vec<Match>,
        data: &str,
        start: usize,
    ) -> Option<ParseResponse> {
        // push newline to avoid matches across multiple keys
        self.str_buf.push('\n');
        let buf_start = self.str_buf.len();
        self.str_buf.push_str(data);

        // create mapping
        self.idx_map.push_back(SegmentMap {
            original: start as u64,
            buffered: (buf_start as u64, self.str_buf.len() as u64),
        });

        if self.str_buf.len() >= Self::BATCH_SIZE_LIMIT {
            self.match_batch(matches)
        } else {
            None
        }
    }

    /// match on the current batch, appending matches to `matches` then resetting the batch state
    fn match_batch(&mut self, matches: &mut Vec<Match>) -> Option<ParseResponse> {
        // TODO: logging displays the uncorrected indexes.
        //      not sure how to get around this without using a near-duplicate custom matching function
        // populate `match_buf` with matches
        let match_result =
            self.matcher
                .do_matching(0, 0, &self.str_buf, &mut self.match_buf, self.performance);

        // sort matches so that we check indexes in order.
        // this allows us to ignore previously checked segments when searching
        self.match_buf.sort_by(|a, b| {
            a.global_start_position
                .unwrap()
                .cmp(&b.global_start_position.unwrap())
        });

        for m in &mut self.match_buf {
            let match_start = m.global_start_position.unwrap();

            // advance to the currently matched segment
            let skip_n = self
                .idx_map
                .iter()
                .position(|o| match_start >= o.buffered.0 && match_start < o.buffered.1)
                .unwrap();
            self.idx_map.rotate_left(skip_n);
            let offset = &self.idx_map[0];

            // restore index of match
            m.global_start_position = Some(offset.original + (match_start - offset.buffered.0));
        }

        // store matches and clear buffers
        matches.append(&mut self.match_buf);
        self.idx_map.clear();
        self.str_buf.clear();
        (match_result == ParseResponse::Block).then_some(match_result)
    }
}

pub struct JsonParser;

#[async_trait::async_trait(?Send)]
impl Parser for JsonParser {
    async fn parse(
        &self,
        policy: &Policy,
        body: &mut PipeReader,
        configuration: ParserConfiguration<'_>,
        matches: &mut Vec<Match>,
        performance: &PerformanceMonitor,
    ) -> Result<ParseResponse> {
        let (key_matcher, value_matcher) = prepare_match_state(policy, &configuration);
        let mut key_matcher = JsonMatcher::new(key_matcher, performance);
        let mut value_matcher = JsonMatcher::new(value_matcher, performance);

        let mut key_matches = Vec::new();
        parse::parse_json(
            body,
            |key, start, _| key_matcher.process(&mut key_matches, &key, start),
            |value, start, _| value_matcher.process(matches, &value, start),
        )
        .await?;
        // perform final match on any pending batches
        key_matcher.match_batch(&mut key_matches);
        value_matcher.match_batch(matches);
        matches.append(&mut key_matches);

        Ok(ParseResponse::Continue)
    }
}
