use std::{borrow::Cow, sync::Arc};

use indexmap::IndexMap;
use leakpolicy::{ContentType, PathConfiguration, Policy, PolicyAction};
use log::warn;

use crate::evaluator::{self, CategoryMatch, MatcherMetadata, MatcherState};

pub mod html;
// pub mod jpeg;
pub mod json;

pub mod grpc;

#[allow(dead_code)]
#[derive(PartialEq)]
pub enum ParseResponse {
    Continue,
    Block,
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
    configuration: &'a IndexMap<Arc<String>, PathConfiguration>,
    content_type: ContentType,
) -> MatcherState<'a> {
    let mut match_state = MatcherState::default();

    for (category_name, action) in configuration {
        if !action.category_config.content_types.is_empty() {
            if !action.category_config.content_types.contains(&content_type) {
                continue;
            }
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
            &policy,
            &**category_name,
            &mut match_state,
            &metadata,
            &action.category_config.ignore,
        );
    }

    match_state
}
