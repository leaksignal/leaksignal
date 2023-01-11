use leakpolicy::{ContentType, EndpointContext, MatchContext, Policy, PolicyAction};

use crate::{
    evaluator::{self, MatcherMetadata, MatcherState},
    parsers::ParserConfiguration,
    perf::PerformanceMonitor,
    Header, HttpParser, ParseResponse, ParsedHeaderMatches, ParsedMatches,
};

pub fn prepare_match_state<'a>(
    policy: &'a Policy,
    configuration: &ParserConfiguration<'a>,
    content_type: ContentType,
    header_name: &str,
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

        // if header contexts were defined but no context matches the header name then dont match category
        let header_context = &action.category_config.contexts;
        if header_context
            .iter()
            .any(|c| matches!(c, MatchContext::Header(_)))
            && !header_context
                .iter()
                .any(|c| matches!(c, MatchContext::Header(s) if s == header_name))
        {
            continue;
        }

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
            true,
        );
    }

    match_state
}

struct HeaderOutputData {
    pub response: ParseResponse,
    pub matches: Vec<ParsedHeaderMatches>,
}

impl Default for HeaderOutputData {
    fn default() -> Self {
        Self {
            response: ParseResponse::Continue,
            matches: Default::default(),
        }
    }
}

impl<'a> HttpParser<'a> {
    fn parse_headers(
        &self,
        headers: impl Iterator<Item = Header>,
        config: &ParserConfiguration,
    ) -> HeaderOutputData {
        let mut output = HeaderOutputData::default();
        for header in headers {
            let response_header_start = self.config.timestamp_source.epoch_ns();
            let mut matches = vec![];
            let performance = PerformanceMonitor::new(self.config.timestamp_source.clone());

            let Some(header_value) = header.value else {
                // if no body then dont bother matching
                output.matches.push(ParsedHeaderMatches {
                    name: header.name,
                    matches: ParsedMatches {
                        time_parse_end: self.config.timestamp_source.epoch_ns(),
                        time_parse_start: response_header_start,
                        matches,
                        body_size: 0,
                        body: None,
                        category_performance_us: performance.into_inner(),
                    },
                });
                continue;
            };

            let matcher = prepare_match_state(
                &self.policy,
                config,
                self.request_description.content_type,
                &header.name.to_lowercase(),
            );
            let resp = matcher.do_matching(0, 0, &header_value, &mut matches, &performance);

            let response_header_end = self.config.timestamp_source.epoch_ns();
            output.matches.push(ParsedHeaderMatches {
                name: header.name,
                matches: ParsedMatches {
                    time_parse_end: response_header_end,
                    time_parse_start: response_header_start,
                    matches,
                    body_size: header_value.len() as u64,
                    body: Some(header_value.into_bytes()),
                    category_performance_us: performance.into_inner(),
                },
            });

            if resp == ParseResponse::Block {
                output.response = resp;
                break;
            }
        }
        output
    }

    pub fn parse_request_headers(
        &mut self,
        headers: impl Iterator<Item = Header>,
    ) -> Option<ParseResponse> {
        let config = ParserConfiguration {
            categories: &self.path_policy.as_ref()?.configuration,
            active_context: EndpointContext::RequestHeader,
        };
        let output = self.parse_headers(headers, &config);
        self.request_header_output = output.matches;
        Some(output.response)
    }

    pub fn parse_response_headers(
        &mut self,
        headers: impl Iterator<Item = Header>,
    ) -> Option<ParseResponse> {
        let config = ParserConfiguration {
            categories: &self.path_policy.as_ref()?.configuration,
            active_context: EndpointContext::ResponseHeader,
        };
        let output = self.parse_headers(headers, &config);
        self.response_header_output = output.matches;
        Some(output.response)
    }
}
