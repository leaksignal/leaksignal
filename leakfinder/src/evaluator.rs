use std::collections::{BTreeMap, HashSet};

use leakpolicy::{CorrelateInterest, DataReportStyle, MatchGroup};
use log::{debug, error, info, warn};
use regex::Regex;
use smallvec::SmallVec;

use crate::{
    parsers::ParseResponse,
    perf::{PerformanceHandle, PerformanceMonitor},
    policy::{evaluate_report_style, Category, Policy, PolicyAction},
    Match,
};

#[derive(Clone, PartialEq, Eq)]
pub struct CategoryMatch<'a> {
    pub start: usize,
    pub length: usize,
    pub value: &'a str,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CategoryPreparedMatch<'a> {
    pub metadata: &'a MatcherMetadata,
    pub start: usize,
    pub length: usize,
}

pub struct MatchRegex<'a> {
    metadata: MatcherMetadata,
    regex: &'a Regex,
    regex_strip: usize,
    ignore: SmallVec<[&'a HashSet<String>; 2]>,
}

pub struct MatchRaw<'a> {
    metadata: MatcherMetadata,
    raw: &'a str,
    case_insensitive: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MatcherMetadata {
    pub policy_path: String,
    pub category_name: String,
    pub action: PolicyAction,
    pub local_report_style: DataReportStyle,
    pub correlation: Option<CorrelationState>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CorrelationState {
    pub correlation_index: usize,
    pub max_distance: usize,
    pub is_second: bool,
    pub interest: CorrelateInterest,
}

#[derive(Default)]
pub struct MatcherState<'a> {
    correlation_index: usize,
    regexes: Vec<MatchRegex<'a>>,
    raws: Vec<MatchRaw<'a>>,
}

fn prepare_match_group<'a>(
    match_group: &'a MatchGroup,
    state: &mut MatcherState<'a>,
    metadata: &MatcherMetadata,
    extra_ignore: &'a HashSet<String>,
    use_multiline: bool,
) {
    let MatchGroup {
        raw,
        regexes,
        regex_strip,
        direct,
        ignore,
        case_insensitive,
    } = match_group;

    for raw in raw {
        state.raws.push(MatchRaw {
            metadata: metadata.clone(),
            raw,
            case_insensitive: *case_insensitive,
        });
    }

    for regex in regexes {
        state.regexes.push(MatchRegex {
            metadata: metadata.clone(),
            regex: if use_multiline {
                &regex.multiline
            } else {
                &regex.original
            },
            regex_strip: *regex_strip,
            ignore: smallvec::smallvec![extra_ignore, ignore],
        });
    }
    for _ in direct {
        todo!("direct matching implementation")
    }
}

pub fn prepare_matches<'a>(
    policy: &'a Policy,
    category_name: &'a str,
    state: &mut MatcherState<'a>,
    metadata: &MatcherMetadata,
    extra_ignore: &'a HashSet<String>,
    use_multiline: bool,
) {
    let category = match policy.categories.get(category_name) {
        Some(x) => x,
        None => {
            error!("invalid category in config: {}", category_name);
            return;
        }
    };

    match category {
        Category::Matchers { match_group } => {
            prepare_match_group(match_group, state, metadata, extra_ignore, use_multiline);
        }
        Category::Correlate {
            group1,
            group2,
            interest,
            max_distance,
        } => {
            let group1 = match group1.match_group(policy) {
                Some(x) => x,
                None => {
                    warn!("correlate match group for '{category_name}' is missing a group1");
                    return;
                }
            };
            let group2 = match group2.match_group(policy) {
                Some(x) => x,
                None => {
                    warn!("correlate match group for '{category_name}' is missing a group2");
                    return;
                }
            };
            let correlation_index = state.correlation_index;
            state.correlation_index += 1;
            let mut metadata = metadata.clone();
            metadata.correlation = Some(CorrelationState {
                correlation_index,
                max_distance: *max_distance,
                is_second: false,
                interest: *interest,
            });
            prepare_match_group(group1, state, &metadata, extra_ignore, use_multiline);
            metadata.correlation = Some(CorrelationState {
                correlation_index,
                max_distance: *max_distance,
                is_second: true,
                interest: *interest,
            });
            prepare_match_group(group2, state, &metadata, extra_ignore, use_multiline);
        }
        Category::Rematch { .. } => {
            error!("rematch in prepared evaluation not implemented");
        }
    }
}

impl<'a> MatcherState<'a> {
    #[cfg(test)]
    pub fn push_raw(&mut self, name: &str, value: &'a str) {
        self.raws.push(MatchRaw {
            metadata: MatcherMetadata {
                policy_path: "".to_string(),
                category_name: name.to_string(),
                action: PolicyAction::Alert,
                local_report_style: DataReportStyle::Raw,
                correlation: None,
            },
            raw: value,
            case_insensitive: true,
        });
    }

    pub fn evaluate(
        &self,
        source: &str,
        performance: &PerformanceMonitor,
    ) -> Vec<CategoryPreparedMatch> {
        let mut matches = vec![];
        let mut handle: Option<PerformanceHandle> = None;
        for raw in &self.raws {
            let metadata = &raw.metadata;
            let case_insensitive = raw.case_insensitive;
            let raw = raw.raw;
            if source.len() < raw.len() {
                continue;
            }
            if let Some(handle) = handle.as_mut() {
                handle.chain(&metadata.category_name);
            } else {
                handle = Some(performance.measure(&metadata.category_name));
            }
            let mut source_iter = source.char_indices().peekable();
            while let Some((i, _)) = source_iter.next() {
                if i > source.len() - raw.len() {
                    break;
                }
                let last_source_byte = source.as_bytes()[i + raw.len() - 1];
                let last_raw_byte = raw.as_bytes()[raw.len() - 1];
                let last_char_matches = if case_insensitive {
                    if last_source_byte < 128 && last_raw_byte < 128 {
                        let source_char =
                            char::from_u32(last_source_byte as u32).expect("ascii is not utf8?");
                        let raw_char =
                            char::from_u32(last_raw_byte as u32).expect("ascii is not utf8?");
                        source_char.to_ascii_lowercase() == raw_char.to_ascii_lowercase()
                    } else {
                        // do a full unicode check
                        true
                    }
                } else {
                    last_source_byte == last_raw_byte
                };
                if last_char_matches {
                    let matches_completely = if case_insensitive {
                        source
                            .get(i..i + raw.len())
                            .map(|x| x.eq_ignore_ascii_case(raw))
                            .unwrap_or(false)
                    } else {
                        source[i..].starts_with(raw)
                    };
                    if matches_completely {
                        matches.push(CategoryPreparedMatch {
                            start: i,
                            length: raw.len(),
                            metadata,
                        });
                        let target = i + raw.len();
                        while let Some((i, _)) = source_iter.peek() {
                            if *i < target {
                                source_iter.next();
                                continue;
                            }
                            break;
                        }
                    }
                }
            }
        }

        for regex in &self.regexes {
            if let Some(handle) = handle.as_mut() {
                handle.chain(&regex.metadata.category_name);
            } else {
                handle = Some(performance.measure(&regex.metadata.category_name));
            }
            for matching in regex.regex.find_iter(source) {
                if regex.ignore.iter().any(|x| x.contains(matching.as_str())) {
                    continue;
                }
                let start = matching.start() + regex.regex_strip;
                let length = matching.end().saturating_sub(start + regex.regex_strip);
                matches.push(CategoryPreparedMatch {
                    metadata: &regex.metadata,
                    start,
                    length,
                });
            }
        }

        matches
    }
}

impl<'a> MatcherState<'a> {
    pub fn do_matching(
        &self,
        offset: usize,
        minimum_end_index: usize,
        body: &str,
        matches: &mut Vec<Match>,
        performance: &PerformanceMonitor,
    ) -> ParseResponse {
        let local_matches = self.evaluate(body, performance);

        let mut correlated_matches_first: BTreeMap<usize, SmallVec<[CategoryPreparedMatch; 2]>> =
            BTreeMap::new();
        let mut correlated_matches_second: BTreeMap<usize, SmallVec<[CategoryPreparedMatch; 2]>> =
            BTreeMap::new();

        for matching in local_matches {
            if matching.start + offset + matching.length <= minimum_end_index {
                debug!("skipping duplicated match");
                continue;
            }
            if let Some(correlation) = matching.metadata.correlation.as_ref() {
                if correlation.is_second {
                    &mut correlated_matches_second
                } else {
                    &mut correlated_matches_first
                }
                .entry(correlation.correlation_index)
                .or_default()
                .push(matching);
                continue;
            }
            let matched_value = evaluate_report_style(
                matching.metadata.local_report_style,
                &body[matching.start..matching.start + matching.length],
            );
            info!(
                "matched {} @ {}-{} -> {:?}: '{}'",
                matching.metadata.category_name,
                matching.start + offset,
                matching.start + offset + matching.length,
                matching.metadata.action,
                matched_value.as_deref().unwrap_or_default()
            );
            matches.push(Match {
                category_name: matching.metadata.category_name.to_string(),
                global_start_position: Some(matching.start as u64 + offset as u64),
                global_length: Some(matching.length as u64),
                matcher_path: matching.metadata.policy_path.clone(),
                matched_value,
            });
        }
        for (index, mut group1) in correlated_matches_first.into_iter() {
            if let Some(mut group2) = correlated_matches_second.remove(&index) {
                group1.sort_by_key(|x| x.start);
                group2.sort_by_key(|x| x.start);
                // avoids overlapping correlations within a group
                let mut continuity_index = 0usize;

                let correlation = group1[0].metadata.correlation.unwrap();
                let distance = correlation.max_distance;
                let mut group2_index = 0usize;
                'outer: for group1_item in group1 {
                    let group1_end = group1_item.start + group1_item.length;
                    if group1_end + offset <= minimum_end_index {
                        debug!("skipping duplicated match (group1)");
                        continue;
                    }
                    if group1_item.start < continuity_index {
                        continue;
                    }
                    while let Some(group2_item) = group2.get(group2_index) {
                        if group2_item.start + group2_item.length + offset <= minimum_end_index {
                            debug!("skipping duplicated match (group2)");
                            continue;
                        }
                        if group2_item.start < continuity_index {
                            group2_index += 1;
                            continue;
                        }

                        let start = group2_item.start.saturating_sub(distance);
                        let end = (group2_item.start + group2_item.length).saturating_add(distance);
                        if group1_end < start {
                            continue 'outer;
                        }
                        if group1_item.start > end {
                            group2_index += 1;
                            continue;
                        }
                        group2_index += 1;
                        let total_start = group1_item.start.min(group2_item.start);
                        let total_end = group1_end.max(group2_item.start + group2_item.length);
                        continuity_index = total_end;
                        let (emit_start, emit_end, emit_report_style) = match correlation.interest {
                            CorrelateInterest::Group1 => (
                                group1_item.start,
                                group1_end,
                                group1_item.metadata.local_report_style,
                            ),
                            CorrelateInterest::Group2 => (
                                group2_item.start,
                                group2_item.start + group2_item.length,
                                group2_item.metadata.local_report_style,
                            ),
                            CorrelateInterest::All => (
                                total_start,
                                total_end,
                                group1_item
                                    .metadata
                                    .local_report_style
                                    .stricter(group2_item.metadata.local_report_style),
                            ),
                        };
                        let matched_value =
                            evaluate_report_style(emit_report_style, &body[emit_start..emit_end]);
                        info!(
                            "matched correlate {} @ {}-{} -> {:?}: '{}'",
                            group1_item.metadata.category_name,
                            total_start + offset,
                            total_end + offset,
                            group1_item.metadata.action,
                            matched_value.as_deref().unwrap_or_default()
                        );
                        matches.push(Match {
                            category_name: group1_item.metadata.category_name.to_string(),
                            global_start_position: Some(emit_start as u64 + offset as u64),
                            global_length: Some((emit_end - emit_start) as u64),
                            matcher_path: group1_item.metadata.policy_path.clone(),
                            matched_value,
                        });
                        break;
                    }
                }
            }
        }

        // if is_block {
        //     return ParseResponse::Block;
        // }

        ParseResponse::Continue
    }
}
