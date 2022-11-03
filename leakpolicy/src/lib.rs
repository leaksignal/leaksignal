use std::{
    collections::{BTreeMap, HashSet},
    sync::Arc,
};

use anyhow::Result;
use fancy_regex::Regex;
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};

mod path_glob;
pub use path_glob::PathGlob;
use serde_single_or_vec2::SingleOrVec;

mod regex_serde {
    use std::borrow::Cow;

    use fancy_regex::Regex;
    use serde::{de::Unexpected, Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(regex: &Regex, serializer: S) -> Result<S::Ok, S::Error> {
        regex.as_str().serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Regex, D::Error> {
        let raw: Cow<'de, str> = Deserialize::deserialize(deserializer)?;
        Regex::new(&*raw)
            .map_err(|e| serde::de::Error::invalid_value(Unexpected::Str(&*raw), &&*e.to_string()))
    }
}

pub fn parse_policy(policy: &str) -> Result<Policy> {
    let parsed: Policy = match serde_yaml::from_str(policy) {
        Ok(x) => x,
        Err(e) => {
            log::debug!("bad leakpolicy:\n{}", policy);
            return Err(e.into());
        }
    };

    // recur_fillin_endpoint(&mut parsed.root_endpoint, "/");
    Ok(parsed)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegexWrapper(#[serde(with = "regex_serde")] pub Regex);

impl PartialEq for RegexWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_str() == other.0.as_str()
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyAction {
    Ignore,
    Alert,
    // Mask {
    //     #[serde(default)]
    //     mask_replacement: String,
    // },
    Block,
}

impl Default for PolicyAction {
    fn default() -> Self {
        PolicyAction::Alert
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    #[serde(alias = "Json")]
    Json,
    #[serde(alias = "Html")]
    Html,
    Grpc,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MatchContext {
    Keys,
    Values,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct AlertConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_request: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_5min_by_ip: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_5min_by_token: Option<usize>,
}

impl AlertConfig {
    pub fn is_empty(&self) -> bool {
        self.per_request.is_none()
            && self.per_5min_by_ip.is_none()
            && self.per_5min_by_token.is_none()
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "report_style")]
pub enum DataReportStyle {
    Raw,
    PartialSha256 {
        report_bits: usize,
    },
    Sha256,
    #[default]
    None,
}

impl DataReportStyle {
    pub fn stricter(self, other: Self) -> Self {
        match (self, other) {
            (DataReportStyle::None, _) | (_, DataReportStyle::None) => DataReportStyle::None,
            (
                DataReportStyle::PartialSha256 {
                    report_bits: report_bits1,
                },
                DataReportStyle::PartialSha256 {
                    report_bits: report_bits2,
                },
            ) => DataReportStyle::PartialSha256 {
                report_bits: report_bits1.min(report_bits2),
            },
            (DataReportStyle::PartialSha256 { report_bits }, _)
            | (_, DataReportStyle::PartialSha256 { report_bits }) => {
                DataReportStyle::PartialSha256 { report_bits }
            }
            (DataReportStyle::Sha256, _) | (_, DataReportStyle::Sha256) => DataReportStyle::Sha256,
            _ => DataReportStyle::Raw,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ConfiguredPolicyAction {
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub action: Option<PolicyAction>,
    /// if empty, no limitation
    /// if present, it's a whitelist
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content_types: SingleOrVec<'static, ContentType>,
    /// if empty, no limitation
    /// if present, describes a whitelist within the structure of a response that this group applies to
    /// interpretation is define by the content_type
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contexts: SingleOrVec<'static, MatchContext>,
    #[serde(default, skip_serializing_if = "AlertConfig::is_empty")]
    pub alert: AlertConfig,
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub ignore: HashSet<String>,
    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub report_style: Option<DataReportStyle>,
}

fn is_zero(x: &usize) -> bool {
    *x == 0
}

fn is_false(x: &bool) -> bool {
    !*x
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MatchGroup {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raw: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub regexes: Vec<RegexWrapper>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub regex_strip: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub direct: Vec<String>,
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub ignore: HashSet<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub case_insensitive: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum MatchGroupRef {
    Inline {
        #[serde(flatten)]
        match_group: MatchGroup,
    },
    Ref {
        name: String,
    },
}

impl MatchGroupRef {
    pub fn match_group<'a>(&'a self, policy: &'a Policy) -> Option<&'a MatchGroup> {
        match self {
            MatchGroupRef::Inline { match_group } => Some(match_group),
            MatchGroupRef::Ref { name } => match policy.categories.get(name)? {
                Category::Matchers { match_group } => Some(match_group),
                _ => None,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CorrelateInterest {
    Group1,
    Group2,
    #[default]
    All,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Category {
    Matchers {
        #[serde(flatten)]
        match_group: MatchGroup,
    },
    Correlate {
        group1: MatchGroupRef,
        group2: MatchGroupRef,
        #[serde(default)]
        interest: CorrelateInterest,
        max_distance: usize,
    },
    Rematch {
        target: MatchGroupRef,
        rematcher: MatchGroupRef,
    },
    // Jpeg {
    //     /// https://docs.rs/kamadak-exif/0.5.4/src/exif/tag.rs.html#252
    //     exif_tags: Vec<String>,
    //     drop_xmp: Option<bool>,
    // },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TokenExtractionSite {
    #[serde(rename = "request")]
    Request,
    #[serde(rename = "request_cookie")]
    RequestCookie,
    #[serde(rename = "response")]
    Response,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TokenExtractionConfig {
    /// location of extraction
    pub location: TokenExtractionSite,
    /// case-insensitive header name for token extraction
    pub header: String,
    /// if it has one or more capture groups, the first capture group is returned
    /// otherwise, the entire match is returned
    pub regex: Option<RegexWrapper>,
    /// if true, the token is SHA-256 hashed
    #[serde(default)]
    pub hash: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EndpointConfig {
    pub matches: SingleOrVec<'static, PathGlob>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub config: IndexMap<Arc<String>, Arc<ConfiguredPolicyAction>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_extractor: Option<Arc<TokenExtractionConfig>>,
    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub report_style: Option<DataReportStyle>,
}

fn collected_request_headers_default() -> IndexSet<String> {
    [
        ":path",
        ":method",
        ":authority",
        ":scheme",
        "accept",
        "accept-encoding",
        "accept-language",
        "cache-control",
        "referer",
        "user-agent",
        "x-request-id",
        "x-forwarded-for",
        "content-type",
        "grpc-encoding",
        "grpc-accept-encoding",
        "x-envoy-peer-metadata",
        "x-envoy-peer-metadata-id",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn collected_response_headers_default() -> IndexSet<String> {
    [
        ":status",
        "content-encoding",
        "content-type",
        "date",
        "server",
        "vary",
        "via",
        "grpc-encoding",
        "grpc-accept-encoding",
        "x-envoy-peer-metadata",
        "x-envoy-peer-metadata-id",
        "grpc-status",
        "grpc-message",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn default_global_report_style() -> DataReportStyle {
    DataReportStyle::Raw
}

fn report_style_is_default(style: &DataReportStyle) -> bool {
    *style == DataReportStyle::Raw
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Policy {
    pub categories: IndexMap<String, Category>,
    /// we apply all matching endpoint configurations,
    /// with the most specific endpoint configuration taking precedence.
    /// The super-root endpoint config has all categories on alert.
    pub endpoints: Vec<EndpointConfig>,
    /// values will be omitted for headers not in this list
    #[serde(default = "collected_request_headers_default")]
    pub collected_request_headers: IndexSet<String>,
    /// values will be omitted for headers not in this list
    #[serde(default = "collected_response_headers_default")]
    pub collected_response_headers: IndexSet<String>,
    /// [0, 1]
    #[serde(default)]
    pub body_collection_rate: f64,
    #[serde(default)]
    #[deprecated = "use report_style"]
    pub collect_matched_values: bool,
    #[serde(
        default = "default_global_report_style",
        skip_serializing_if = "report_style_is_default"
    )]
    pub report_style: DataReportStyle,
}

pub struct PathPolicy {
    pub policy_path: String,
    pub configuration: IndexMap<Arc<String>, PathConfiguration>,
    pub token_extractor: Option<Arc<TokenExtractionConfig>>,
}

pub struct PathConfiguration {
    pub matcher_path: String,
    pub category_config: Arc<ConfiguredPolicyAction>,
    pub report_style: DataReportStyle,
}

impl Policy {
    pub fn get_path_config<'a>(&'a self, path: &str) -> PathPolicy {
        let path = if let Some((left, _)) = path.split_once('?') {
            left
        } else {
            path
        };

        let components = path
            .split('/')
            .map(|x| x.trim())
            .filter(|x| !x.is_empty())
            .collect::<Vec<_>>();
        let mut output = IndexMap::new();

        let mut policy_paths: BTreeMap<&PathGlob, Vec<&EndpointConfig>> = BTreeMap::new();

        for endpoint in self.endpoints.iter() {
            for path in &*endpoint.matches {
                if path.matches_components(components.iter().copied()) {
                    policy_paths.entry(path).or_default().push(endpoint);
                }
            }
        }

        let mut token_extractor = None;

        for (path, configs) in policy_paths.iter().rev() {
            for config in configs {
                for (category, config) in &config.config {
                    output.insert(
                        category.clone(),
                        PathConfiguration {
                            matcher_path: (*path).to_string(),
                            category_config: config.clone(),
                            report_style: config
                                .report_style
                                .or(config.report_style)
                                .unwrap_or(self.report_style),
                        },
                    );
                }
                if let Some(extractor) = &config.token_extractor {
                    token_extractor = Some(extractor.clone());
                }
            }
        }

        PathPolicy {
            policy_path: policy_paths
                .iter()
                .next()
                .map(|(x, _)| x.to_string())
                .unwrap_or_else(|| String::new()),
            configuration: output,
            token_extractor,
        }
    }
}
