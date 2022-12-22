use std::{borrow::Cow, io::Write, sync::Arc, task::Poll};

use futures::{pin_mut, task::waker, FutureExt};
use indexmap::IndexMap;
use leakpolicy::{ContentType, EndpointContext, PathConfiguration, Policy, PolicyAction};
use log::warn;

use crate::{
    evaluator::{self, CategoryMatch, MatcherMetadata, MatcherState},
    perf::PerformanceMonitor,
    pipe::{pipe, DummyWaker, PipeReader},
    Match,
};
use anyhow::Result;

pub mod plaintext;
// pub mod jpeg;
pub mod json;

pub mod grpc;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq)]
pub enum ParseResponse {
    Continue,
    Block,
}

pub struct ParserConfiguration<'a> {
    pub categories: &'a IndexMap<Arc<String>, PathConfiguration>,
    pub active_context: EndpointContext,
}

#[allow(dead_code)]
fn replace_matches<'a>(
    body: &'a str,
    matches: &[CategoryMatch],
    expected_length: usize,
    replace_with: &str,
) -> Cow<'a, str> {
    if matches.is_empty() {
        return Cow::Borrowed(body);
    }
    let mut output = String::with_capacity(expected_length);
    let mut body_index = 0usize;
    for policy_match in matches {
        if body_index > policy_match.start {
            warn!("overlapping matches detected!");
            body_index = policy_match.start;
        }
        output.push_str(&body[body_index..policy_match.start]);
        output.push_str(replace_with);
        body_index = policy_match.start + policy_match.length;
    }
    output.push_str(&body[body_index..]);
    Cow::Owned(output)
}

/// prepares a match for a content type, doesn't handle contexts
fn prepare_match_state<'a>(
    policy: &'a Policy,
    configuration: ParserConfiguration<'a>,
    content_type: ContentType,
) -> MatcherState<'a> {
    let mut match_state = MatcherState::default();

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
            && !action.category_config.content_types.contains(&content_type)
        {
            continue;
        }

        if matches!(
            action.category_config.action.unwrap_or_default(),
            PolicyAction::Ignore
        ) {
            continue;
        }

        // let is_block = matches!(action.action, PolicyAction::Block);

        let metadata = MatcherMetadata {
            policy_path: action.matcher_path.clone(),
            category_name: category_name.to_string(),
            action: action.category_config.action.unwrap_or_default(),
            local_report_style: action.report_style,
            correlation: None,
        };

        evaluator::prepare_matches(
            policy,
            category_name,
            &mut match_state,
            &metadata,
            &action.category_config.ignore,
        );
    }

    match_state
}

#[async_trait::async_trait(?Send)]
pub trait Parser {
    fn parse_sync(
        &self,
        policy: &Policy,
        body: &[u8],
        configuration: ParserConfiguration<'_>,
        matches: &mut Vec<Match>,
        performance: &PerformanceMonitor,
    ) -> Result<ParseResponse> {
        let (mut reader, mut writer) = pipe(0);
        writer.write_all(body)?;
        drop(writer);

        let future = self.parse(policy, &mut reader, configuration, matches, performance);
        pin_mut!(future);
        let waker = waker(Arc::new(DummyWaker));
        let mut context = std::task::Context::from_waker(&waker);
        match future.as_mut().poll_unpin(&mut context) {
            Poll::Ready(Err(e)) => Err(e),
            Poll::Ready(Ok(data)) => Ok(data),
            Poll::Pending => unimplemented!("future did not complete in time"),
        }
    }

    async fn parse(
        &self,
        policy: &Policy,
        body: &mut PipeReader,
        configuration: ParserConfiguration<'_>,
        matches: &mut Vec<Match>,
        performance: &PerformanceMonitor,
    ) -> Result<ParseResponse>;
}
