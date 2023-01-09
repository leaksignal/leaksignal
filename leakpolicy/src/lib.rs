use std::{
    borrow::Cow,
    collections::{BTreeMap, HashSet},
    net::IpAddr,
    ops::Deref,
    str::FromStr,
    sync::Arc,
};

use anyhow::Result;
use indexmap::{IndexMap, IndexSet};
use ipnetwork::IpNetwork;
use regex::{Regex, RegexBuilder};
use serde::{de::Unexpected, Deserialize, Deserializer, Serialize, Serializer};

mod match_rule;
mod path_glob;
pub use match_rule::MatchRule;
pub use path_glob::PathGlob;
use serde_single_or_vec2::SingleOrVec;

pub fn parse_policy(policy: &str) -> Result<Policy> {
    serde_yaml::from_str(policy).map_err(|e| {
        log::debug!("bad leakpolicy:\n{}", policy);
        e.into()
    })

    // recur_fillin_endpoint(&mut parsed.root_endpoint, "/");
}

#[derive(Clone, Debug)]
pub struct RegexWrapper {
    /// the original, unmodified regex
    pub original: Regex,
    /// the same regex as `original` but with multiline mode turned on.
    /// used for json's batched matching
    pub multiline: Regex,
}

impl PartialEq for RegexWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.original.as_str() == other.original.as_str()
    }
}

impl Serialize for RegexWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.original.as_str().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RegexWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let r: Cow<'de, str> = Deserialize::deserialize(deserializer)?;

        Ok(Self {
            original: Regex::new(&r).map_err(|e| {
                serde::de::Error::invalid_value(Unexpected::Str(&r), &e.to_string().deref())
            })?,
            multiline: RegexBuilder::new(&r)
                .multi_line(true)
                .build()
                .map_err(|e| {
                    serde::de::Error::invalid_value(Unexpected::Str(&r), &e.to_string().deref())
                })?,
        })
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    Html,
    Json,
    Grpc,
    UrlEncoded,
    Jpeg,
    Unknown,
}

impl Default for ContentType {
    fn default() -> Self {
        ContentType::Unknown
    }
}

impl FromStr for ContentType {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        // must be infallible
        let value = if let Some((init, _)) = value.split_once(';') {
            init.trim()
        } else {
            value.trim()
        };
        Ok(match value {
            "text/html" => ContentType::Html,
            "application/grpc+proto" | "application/grpc" => ContentType::Grpc,
            "application/x-www-form-urlencoded" => ContentType::UrlEncoded,
            "image/jpg" | "image/jpeg" => ContentType::Jpeg,
            "application/json" => ContentType::Json,
            _ => ContentType::Unknown,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MatchContext {
    Keys,
    Values,
    #[serde(
        deserialize_with = "header_from_string",
        serialize_with = "header_to_string"
    )]
    Header(String),
}

impl MatchContext {
    pub fn header(&self) -> Option<&str> {
        if let Self::Header(s) = self {
            Some(s)
        } else {
            None
        }
    }
}

fn header_from_string<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    String::deserialize(deserializer).map(|s| s.to_lowercase())
}

fn header_to_string<S: Serializer>(v: &str, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&v.to_lowercase())
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
    #[serde(default)]
    pub search: EndpointContext,
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
#[serde(rename_all = "snake_case")]
pub enum TokenExtractionSite {
    Request,
    RequestCookie,
    Response,
}

#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EndpointContext {
    RequestBody,
    RequestHeader,
    ResponseBody,
    ResponseHeader,
    #[default]
    AllBody,
    AllHeader,
    All,
}

impl EndpointContext {
    /// Matches if the `self` EndpointContext contains the `other` EndpointContext
    pub fn match_specific(self, other: EndpointContext) -> bool {
        match (self, other) {
            (x, y) if x == y => true,
            (
                EndpointContext::AllHeader,
                EndpointContext::ResponseHeader | EndpointContext::RequestHeader,
            ) => true,
            (
                EndpointContext::AllBody,
                EndpointContext::ResponseBody | EndpointContext::RequestBody,
            ) => true,
            (EndpointContext::All, _) => true,
            _ => false,
        }
    }
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

#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitGroup {
    Global,
    PerService,
    #[default]
    PerEndpoint,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitBy {
    Ip,
    Token,
    Service,
}

#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitAction {
    Nothing,
    Alert,
    #[default]
    Block,
}

fn default_timespan_secs() -> u64 {
    60
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitFilter {
    Endpoint(SingleOrVec<'static, PathGlob>),
    ExcludeEndpoint(SingleOrVec<'static, PathGlob>),
    PeerService(SingleOrVec<'static, MatchRule>),
    ExcludePeerService(SingleOrVec<'static, MatchRule>),
    LocalService(SingleOrVec<'static, MatchRule>),
    ExcludeLocalService(SingleOrVec<'static, MatchRule>),
    Token(SingleOrVec<'static, MatchRule>),
    ExcludeToken(SingleOrVec<'static, MatchRule>),
    Ip(SingleOrVec<'static, IpNetwork>),
    ExcludeIp(SingleOrVec<'static, IpNetwork>),
    Any(Vec<RateLimitFilter>),
    All(Vec<RateLimitFilter>),
}

impl Default for RateLimitFilter {
    fn default() -> Self {
        Self::All(Vec::default())
    }
}

pub struct RateLimitFilterInput<'a> {
    pub ip: IpAddr,
    pub token: &'a str,
    pub endpoint: &'a str,
    pub peer_service: &'a str,
    pub local_service: &'a str,
}

impl RateLimitFilter {
    pub fn matches(&self, input: &RateLimitFilterInput<'_>) -> bool {
        match self {
            RateLimitFilter::Endpoint(endpoints) => {
                endpoints.iter().any(|x| x.matches(input.endpoint))
            }
            RateLimitFilter::ExcludeEndpoint(endpoints) => {
                !endpoints.iter().any(|x| x.matches(input.endpoint))
            }
            RateLimitFilter::PeerService(matchers) => {
                MatchRule::match_all(input.peer_service, matchers)
            }
            RateLimitFilter::ExcludePeerService(matchers) => {
                !MatchRule::match_all(input.peer_service, matchers)
            }
            RateLimitFilter::LocalService(matchers) => {
                MatchRule::match_all(input.local_service, matchers)
            }
            RateLimitFilter::ExcludeLocalService(matchers) => {
                !MatchRule::match_all(input.local_service, matchers)
            }
            RateLimitFilter::Token(matchers) => MatchRule::match_all(input.token, matchers),
            RateLimitFilter::ExcludeToken(matchers) => !MatchRule::match_all(input.token, matchers),
            RateLimitFilter::Ip(matchers) => matchers.iter().any(|x| x.contains(input.ip)),
            RateLimitFilter::ExcludeIp(matchers) => !matchers.iter().any(|x| x.contains(input.ip)),
            RateLimitFilter::Any(filters) => filters.iter().any(|f| f.matches(input)),
            RateLimitFilter::All(filters) => filters.iter().all(|f| f.matches(input)),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RateLimitConfig {
    #[serde(default)]
    pub grouping: RateLimitGroup,
    pub by: RateLimitBy,
    #[serde(default)]
    pub action: RateLimitAction,
    #[serde(default = "default_timespan_secs")]
    pub timespan_secs: u64,
    pub limit: u64,
    #[serde(default)]
    pub filter: RateLimitFilter,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EndpointConfig {
    pub matches: SingleOrVec<'static, PathGlob>,
    #[serde(default)]
    pub spiffe_id: SingleOrVec<'static, String>,
    #[serde(default)]
    pub search: EndpointContext,
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
        "x-envoy-peer-metadata-id",
        "grpc-status",
        "grpc-message",
        "x-ls-request-id",
        "x-source",
        "x-ls-source",
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

fn default_max_body_collection_mb() -> f64 {
    16.0
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ServicePolicy {
    /// List of services this policy matches for
    pub services: SingleOrVec<'static, MatchRule>,
    /// If specified, these services are disallowed from communicating. Ignored if `whitelist` is nonempty.
    #[serde(default)]
    pub blacklist: Vec<MatchRule>,
    /// If specified, only these services can communicate with this service
    #[serde(default)]
    pub whitelist: Vec<MatchRule>,
    /// If `whitelist` is nonempty, this defaults to `true`. Otherwise, `false`. When `true`, inbound communications from unknown services (no mTLS) is blocked.
    pub block_unknown_services: Option<bool>,
}

impl ServicePolicy {
    pub fn service_matched(&self, service_name: &str) -> bool {
        MatchRule::match_all(service_name, &self.services)
    }

    pub fn block_unknown_services(&self) -> bool {
        self.block_unknown_services
            .unwrap_or(!self.whitelist.is_empty())
    }

    pub fn inbound_allowed(&self, service_name: Option<&str>) -> bool {
        let Some(service_name) = service_name else {
            return !self.block_unknown_services();
        };
        if !self.whitelist.is_empty() {
            MatchRule::match_all(service_name, &self.whitelist)
        } else {
            !MatchRule::match_all(service_name, &self.blacklist)
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Policy {
    pub categories: IndexMap<String, Category>,
    /// we apply all matching endpoint configurations,
    /// with the most specific endpoint configuration taking precedence.
    /// The super-root endpoint config has all categories on alert.
    #[serde(default)]
    pub endpoints: Vec<EndpointConfig>,
    #[serde(default)]
    pub services: Vec<ServicePolicy>,
    /// values will be omitted for headers not in this list
    #[serde(default = "collected_request_headers_default")]
    pub collected_request_headers: IndexSet<String>,
    /// values will be omitted for headers not in this list
    #[serde(default = "collected_response_headers_default")]
    pub collected_response_headers: IndexSet<String>,
    /// [0, 1]
    #[serde(default)]
    pub body_collection_rate: f64,
    #[serde(default = "default_max_body_collection_mb")]
    pub max_body_collection_mb: f64,
    #[serde(default)]
    #[deprecated = "use report_style"]
    pub collect_matched_values: bool,
    #[serde(
        default = "default_global_report_style",
        skip_serializing_if = "report_style_is_default"
    )]
    pub report_style: DataReportStyle,
    #[serde(default)]
    pub blocked_ips: IndexSet<String>,
    #[serde(default)]
    pub blocked_tokens: IndexSet<String>,
    #[serde(default)]
    pub ratelimits: Vec<RateLimitConfig>,
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
    pub search: EndpointContext,
}

impl Policy {
    pub fn get_path_config(&self, path: &str) -> PathPolicy {
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
            for path in endpoint.matches.iter() {
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
                            search: config.search,
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
                .unwrap_or_default(),
            configuration: output,
            token_extractor,
        }
    }
}
