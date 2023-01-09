use std::{fmt::Write as FmtWrite, str::FromStr, sync::Arc};

use anyhow::Result;
use leakpolicy::{ContentType, EndpointContext, RegexWrapper};
use log::info;
use sha2::{Digest, Sha256};

use crate::{
    config::Config,
    match_data::Header,
    policy::{PathPolicy, PolicyRef, TokenExtractionConfig, TokenExtractionSite},
    EvaluationOutput, ParsedHeaderMatches, ParsedMatches,
};

mod headers;
mod response;
pub use response::*;

#[derive(Clone, Copy, PartialEq, Debug)]
enum ContentEncoding {
    Gzip,
    None,
    Unknown,
}

impl Default for ContentEncoding {
    fn default() -> Self {
        ContentEncoding::None
    }
}

impl FromStr for ContentEncoding {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        // must be infallible
        Ok(match s {
            "gzip" => ContentEncoding::Gzip,
            _ => ContentEncoding::Unknown,
        })
    }
}

#[derive(Default)]
struct ContentDescription {
    content_encoding: ContentEncoding,
    content_type: ContentType,
}

pub struct HttpParser<'a> {
    response_description: ContentDescription,
    request_description: ContentDescription,
    path: Option<String>,
    hostname: Option<String>,
    config: &'a Config,
    policy: PolicyRef,
    path_policy: Option<Arc<PathPolicy>>,
    ip: Option<String>,
    token: Option<String>,
    time_request_start: u64,
    time_request_body_start: Option<u64>,
    time_response_start: u64,
    response_header_output: Vec<ParsedHeaderMatches>,
    request_header_output: Vec<ParsedHeaderMatches>,
    response_output: Option<ParsedMatches>,
    request_output: Option<ParsedMatches>,
}

impl Config {
    pub fn http_parser(&self) -> Option<HttpParser<'_>> {
        Some(HttpParser {
            response_description: Default::default(),
            request_description: Default::default(),
            config: self,
            policy: self.policy.policy()?,
            ip: Default::default(),
            token: Default::default(),
            path: Default::default(),
            hostname: Default::default(),
            time_request_start: 0,
            time_response_start: 0,
            time_request_body_start: None,
            path_policy: None,
            response_header_output: Default::default(),
            request_header_output: Default::default(),
            response_output: None,
            request_output: None,
        })
    }
}

fn extract_token_regex(value: &str, regex: Option<&RegexWrapper>, hash: bool) -> Option<String> {
    let value = if let Some(RegexWrapper {
        original: regex, ..
    }) = regex
    {
        let captures = regex.captures(value)?;
        if let Some(captured) = captures.get(1) {
            captured.as_str()
        } else {
            captures.get(0)?.as_str()
        }
    } else {
        value
    };
    if hash {
        let mut out = String::with_capacity(64);
        for byte in Sha256::new().chain_update(value.as_bytes()).finalize() {
            write!(&mut out, "{byte:02X}").ok()?;
        }
        Some(out)
    } else {
        Some(value.to_string())
    }
}

pub struct FullHeader {
    pub name: String,
    pub value: String,
}

impl From<(String, Vec<u8>)> for FullHeader {
    fn from((name, value): (String, Vec<u8>)) -> Self {
        let value = String::from_utf8(value)
            .unwrap_or_else(|e| String::from_utf8_lossy(&e.into_bytes()).into_owned());
        Self { name, value }
    }
}

impl From<(String, String)> for FullHeader {
    fn from((name, value): (String, String)) -> Self {
        Self { name, value }
    }
}

impl<'a> HttpParser<'a> {
    pub fn policy(&self) -> &PolicyRef {
        &self.policy
    }

    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    pub fn with_ip(&mut self, ip: String) {
        self.ip = Some(ip);
    }

    pub fn with_request_headers(
        &mut self,
        headers: impl IntoIterator<Item = FullHeader> + 'a,
    ) -> impl Iterator<Item = Header> + '_ {
        if self.time_request_start == 0 {
            self.time_request_start = self.config.timestamp_source.epoch_ns();
        }

        headers.into_iter().map(|FullHeader { name, value }| {
            match name.as_str() {
                ":path" => self.path = Some(value.clone()),
                ":authority" => self.hostname = Some(value.clone()),
                "content-type" => {
                    self.request_description.content_type = value
                        .parse()
                        .expect("content-type parse failed (impossible)")
                }
                "content-encoding" => {
                    self.request_description.content_encoding = value
                        .parse()
                        .expect("content-encoding parse failed (impossible)")
                }
                _ => {}
            }

            if self.path_policy.is_none() {
                if let (Some(path), Some(hostname)) = (&self.path, &self.hostname) {
                    let full_path = format!("{}{}", hostname, path);
                    self.path_policy = Some(Arc::new(self.policy.get_path_config(&full_path)));
                }
            }
            // token extraction on request body
            if let Some(token_extractor) = self
                .path_policy
                .as_ref()
                .and_then(|x| x.token_extractor.as_deref())
            {
                match token_extractor {
                    TokenExtractionConfig {
                        location: TokenExtractionSite::Request,
                        header,
                        regex,
                        hash,
                    } if &name == header => {
                        self.token = extract_token_regex(&value, regex.as_ref(), *hash);
                    }
                    TokenExtractionConfig {
                        location: TokenExtractionSite::RequestCookie,
                        header,
                        regex,
                        hash,
                    } if name == "cookie" => {
                        for value in value.split("; ") {
                            if let Some((name, value)) = value.split_once('=') {
                                if name == header {
                                    self.token = extract_token_regex(value, regex.as_ref(), *hash);
                                    break;
                                }
                            };
                        }
                    }
                    _ => {}
                }
            }

            // record request headers
            let value = self
                .policy
                .collected_request_headers
                .contains(&name)
                .then_some(value);

            Header { name, value }
        })
    }

    pub fn with_response_headers(
        &mut self,
        headers: impl IntoIterator<Item = FullHeader> + 'a,
    ) -> impl Iterator<Item = Header> + '_ {
        if self.time_response_start == 0 {
            self.time_response_start = self.config.timestamp_source.epoch_ns();
        }

        headers.into_iter().map(|FullHeader { name, value }| {
            // token extraction on response body
            match self
                .path_policy
                .as_ref()
                .and_then(|x| x.token_extractor.as_deref())
            {
                Some(TokenExtractionConfig {
                    location: TokenExtractionSite::Response,
                    header,
                    regex,
                    hash,
                }) if &name == header => {
                    self.token = extract_token_regex(&value, regex.as_ref(), *hash);
                }
                _ => {}
            }

            match name.as_str() {
                "content-type" => {
                    self.response_description.content_type = value
                        .parse()
                        .expect("content-type parse failed (impossible)")
                }
                "content-encoding" => {
                    self.response_description.content_encoding = value
                        .parse()
                        .expect("content-encoding parse failed (impossible)")
                }
                _ => {}
            }

            // record response headers
            let value = self
                .policy
                .collected_response_headers
                .contains(&name)
                .then_some(value);

            Header { name, value }
        })
    }

    pub fn with_response_trailers(
        &mut self,
        headers: impl IntoIterator<Item = FullHeader> + 'a,
    ) -> impl Iterator<Item = Header> + '_ {
        headers.into_iter().map(|h| {
            // record response trailers
            Header {
                value: self
                    .policy
                    .collected_response_headers
                    .contains(&h.name)
                    .then_some(h.value),
                name: h.name,
            }
        })
    }

    pub fn with_request_stream(&mut self) -> Option<BodyContext> {
        self.time_request_body_start = Some(self.config.timestamp_source.epoch_ns());

        Some(BodyContext::spawn(
            self.policy.clone(),
            self.config.timestamp_source.clone(),
            self.path_policy.clone()?,
            EndpointContext::RequestBody,
            self.request_description.content_type,
            self.request_description.content_encoding,
        ))
    }

    pub fn with_response_stream(&mut self) -> Option<BodyContext> {
        Some(BodyContext::spawn(
            self.policy.clone(),
            self.config.timestamp_source.clone(),
            self.path_policy.clone()?,
            EndpointContext::ResponseBody,
            self.response_description.content_type,
            self.response_description.content_encoding,
        ))
    }

    pub fn finish_request_stream(&mut self, data: ParsedMatches) {
        self.request_output = Some(data);
    }

    pub fn finish_response_stream(&mut self, data: ParsedMatches) {
        self.response_output = Some(data);
    }

    pub fn finish(mut self) -> EvaluationOutput {
        let response = self.response_output.take().expect("missing response data");
        let request = self.request_output.unwrap_or_else(|| ParsedMatches {
            matches: vec![],
            body_size: 0,
            body: None,
            category_performance_us: Default::default(),
            time_parse_start: 0,
            time_parse_end: 0,
        });
        for m in response.matches.iter().chain(&request.matches) {
            info!(
                "matched {} @ {}-{} -> {:?}: {:?}",
                m.category_name,
                m.global_start_position.unwrap(),
                m.global_start_position.unwrap() + m.global_length.unwrap(),
                m.matcher_path,
                m.matched_value.as_ref().unwrap()
            );
        }
        for h in self
            .request_header_output
            .iter()
            .chain(&self.response_header_output)
        {
            for m in &h.matches.matches {
                info!(
                    "matched {} in header @ {}:{}-{} -> {:?}: {:?}",
                    m.category_name,
                    h.name,
                    m.global_start_position.unwrap(),
                    m.global_start_position.unwrap() + m.global_length.unwrap(),
                    m.matcher_path,
                    m.matched_value.as_ref().unwrap()
                )
            }
        }
        EvaluationOutput {
            policy_id: self.policy.policy_id().to_string(),
            time_request_start: self.time_request_start,
            time_response_start: self.time_response_start,
            request_headers: self.request_header_output,
            response_headers: self.response_header_output,
            policy_path: self
                .path_policy
                .as_ref()
                .map(|x| x.policy_path.to_string())
                .unwrap_or_default(),
            token: self.token.unwrap_or_default(),
            ip: self.ip.unwrap_or_default(),
            response,
            request,
        }
    }
}
