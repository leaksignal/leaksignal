use std::{collections::HashMap, time::Duration};

use anyhow::{bail, Result};
use leakfinder::{FullHeader, Header, HttpParser, Match, ParseResponse, ResponseBodyContext};
use log::{error, warn};
use prost::Message;
use proxy_wasm::{
    traits::{Context, HttpContext},
    types::{Action, MetricType},
};

use crate::{
    config::{upstream, LEAKSIGNAL_SERVICE_NAME},
    metric::Metric,
    proto::{Header as ProtoHeader, Match as ProtoMatch, MatchDataRequest},
    root::{DYN_ENVIRONMENT, LEAKFINDER_CONFIG},
    GIT_COMMIT,
};

const MATCH_PUSH_TIMEOUT: Duration = Duration::from_secs(5);

impl Into<ProtoHeader> for Header {
    fn into(self) -> ProtoHeader {
        ProtoHeader {
            name: self.name,
            value: self.value,
        }
    }
}

impl Into<ProtoMatch> for Match {
    fn into(self) -> ProtoMatch {
        ProtoMatch {
            category_name: self.category_name,
            global_start_position: self.global_start_position,
            global_length: self.global_length,
            matcher_path: self.matcher_path,
            matched_value: self.matched_value,
        }
    }
}

pub struct HttpResponseContext {
    parser: Option<HttpParser<'static>>,
    connection_info: HashMap<String, String>,
    request_started: bool,
    response_body: Option<ResponseBodyContext>,
}

impl Default for HttpResponseContext {
    fn default() -> Self {
        Self {
            parser: LEAKFINDER_CONFIG.http_parser(),
            connection_info: Default::default(),
            request_started: false,
            response_body: None,
        }
    }
}

impl Context for HttpResponseContext {
    fn on_grpc_call_response(&mut self, _token_id: u32, status_code: u32, _response_size: usize) {
        if status_code != 0 {
            warn!("MatchData upload failed with status_code {status_code}");
        }
    }
}

impl HttpResponseContext {
    fn receive_body_chunk(&mut self, body_size: usize) -> Result<Vec<u8>> {
        let body = match self.get_http_response_body(0, body_size) {
            Some(x) => x,
            None => {
                bail!("missing body for response");
            }
        };
        Ok(body)
    }

    fn parser(&mut self) -> &mut HttpParser<'static> {
        self.parser.as_mut().unwrap()
    }

    fn get_property_parse(&self, name: &str) -> Option<Vec<u8>> {
        self.get_property(name.split('.').collect())
            .map(|x| x.to_vec())
    }

    fn get_property_bool(&self, name: &str) -> Option<bool> {
        let raw = self.get_property_parse(name)?;
        if raw.is_empty() || raw.len() != 1 {
            return None;
        }
        Some(raw[0] != 0)
    }

    fn get_property_string(&self, name: &str) -> Option<String> {
        let raw = self.get_property_parse(name)?;
        Some(String::from_utf8_lossy(&raw[..]).into_owned())
    }
}

const STRING_CONNECTION_PROPERTIES: &[&str] = &[
    "connection.requested_server_name",
    "connection.tls_version",
    "connection.subject_local_certificate",
    "connection.subject_peer_certificate",
    "connection.dns_san_local_certificate",
    "connection.dns_san_peer_certificate",
    "connection.uri_san_local_certificate",
    "connection.uri_san_peer_certificate",
    "upstream.address",
    "upstream.port",
    "upstream.tls_version",
    "upstream.subject_local_certificate",
    "upstream.subject_peer_certificate",
    "upstream.dns_san_local_certificate",
    "upstream.dns_san_peer_certificate",
    "upstream.uri_san_local_certificate",
    "upstream.uri_san_peer_certificate",
];

impl HttpContext for HttpResponseContext {
    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        if self.parser.is_none() {
            warn!("processing request, but no policy loaded");
            return Action::Continue;
        }

        if !self.request_started {
            self.request_started = true;
            if let Some(mtls) = self.get_property_bool("connection.mtls") {
                self.connection_info
                    .insert("mtls".to_string(), mtls.to_string());
            }
            for property in STRING_CONNECTION_PROPERTIES {
                if let Some(value) = self.get_property_string(property) {
                    self.connection_info.insert(property.to_string(), value);
                }
            }

            let mut ip = String::from_utf8_lossy(
                &self
                    .get_property(vec!["source", "address"])
                    .unwrap_or_default()[..],
            )
            .into_owned();
            // remove port number
            if let Some(last_colon) = ip.rfind(':') {
                ip.truncate(last_colon);
            }
            self.parser().with_ip(ip);
        }

        for (name, value) in self.get_http_request_headers_bytes() {
            let value = match String::from_utf8(value) {
                Ok(x) => x,
                Err(e) => String::from_utf8_lossy(&e.into_bytes()[..]).into_owned(),
            };
            self.parser()
                .with_request_headers([FullHeader { name, value }]);
        }

        Action::Continue
    }

    fn on_http_response_headers(&mut self, _: usize, _end_of_stream: bool) -> Action {
        if self.parser.is_none() {
            return Action::Continue;
        }

        self.set_http_response_header("content-length", None);
        for (name, value) in self.get_http_response_headers_bytes() {
            let value = match String::from_utf8(value) {
                Ok(x) => x,
                Err(e) => String::from_utf8_lossy(&e.into_bytes()[..]).into_owned(),
            };
            self.parser()
                .with_response_headers([FullHeader { name, value }]);
        }

        Action::Continue
    }

    fn on_http_response_trailers(&mut self, _: usize) -> Action {
        if self.parser.is_none() {
            return Action::Continue;
        }

        for (name, value) in self.get_http_response_trailers_bytes() {
            let value = match String::from_utf8(value) {
                Ok(x) => x,
                Err(e) => String::from_utf8_lossy(&e.into_bytes()[..]).into_owned(),
            };
            self.parser()
                .with_response_trailers([FullHeader { name, value }]);
        }

        Action::Continue
    }

    fn on_http_response_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        if self.parser.is_none() {
            return Action::Continue;
        }

        if self.response_body.is_none() {
            self.response_body = self.parser().with_response_stream();
            if self.response_body.is_none() {
                warn!("unable to create response stream, were we missing :path or :authority?");
                return Action::Continue;
            }
        }

        if body_size > 0 {
            let parse_response = match self
                .receive_body_chunk(body_size)
                .and_then(|body| self.response_body.as_mut().unwrap().receive_chunk(body))
            {
                Err(e) => {
                    error!("failed to decode body: {e:?}");
                    return Action::Continue;
                }
                Ok(x) => x,
            };

            match parse_response {
                ParseResponse::Block => {
                    warn!("blocking request");
                    self.set_http_response_body(0, body_size, &[]);
                }
                ParseResponse::Continue => (),
            }
        }

        if end_of_stream {
            let response = match self.response_body.take().unwrap().end_stream() {
                Err(e) => {
                    error!("failed to decode body at end of stream: {e:?}");
                    return Action::Continue;
                }
                Ok(x) => x,
            };
            self.parser().finish_response_stream(response.matches);
            let policy_id = self.parser().policy().policy_id().to_string();
            let output = self.parser.take().unwrap().finish();

            let policy_path = &*output.policy_path;

            let mut match_counts: HashMap<&str, i64> = HashMap::new();
            for matching in &output.response.matches {
                *match_counts.entry(&*matching.category_name).or_default() += 1;
            }

            for (category_name, count) in match_counts {
                let metric = Metric::lookup_or_define(
                    format!("ls.{policy_path}.resp.{category_name}.count"),
                    MetricType::Counter,
                );
                metric.increment(count);
            }
            for (category_name, us_taken) in &output.response.category_performance_us {
                let metric = Metric::lookup_or_define(
                    format!("ls.{policy_path}.resp.{category_name}.us_taken"),
                    MetricType::Histogram,
                );
                metric.set_value(*us_taken as u64);
            }

            if let Some(upstream) = upstream() {
                let packet = MatchDataRequest {
                    api_key: upstream.api_key.clone(),
                    deployment_name: upstream.deployment_name.clone(),
                    policy_id,
                    highest_action_taken: crate::proto::Action::Alert as i32,
                    time_request_start: output.time_request_start,
                    time_response_start: output.time_response_start,
                    time_response_body_start: output.time_response_body_start,
                    time_response_body_end: output.response.time_parse_end,
                    request_headers: output.request_headers.into_iter().map(Into::into).collect(),
                    response_headers: output
                        .response_headers
                        .into_iter()
                        .map(Into::into)
                        .collect(),
                    matches: output
                        .response
                        .matches
                        .into_iter()
                        .map(Into::into)
                        .collect(),
                    body_size: output.response.body_size,
                    body: output.response.body,
                    policy_path: output.policy_path,
                    commit: GIT_COMMIT.to_string(),
                    token: output.token,
                    ip: output.ip,
                    category_performance_us: output.response.category_performance_us,
                    connection_info: std::mem::take(&mut self.connection_info),
                    environment: DYN_ENVIRONMENT
                        .load()
                        .iter()
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .collect(),
                };

                let emitted_packet = packet.encode_to_vec();

                if let Err(e) = self.dispatch_grpc_call(
                    unsafe { std::str::from_utf8_unchecked(&upstream.service_definition[..]) },
                    LEAKSIGNAL_SERVICE_NAME,
                    "MatchData",
                    vec![],
                    Some(&emitted_packet[..]),
                    MATCH_PUSH_TIMEOUT,
                ) {
                    error!("failed to upstream match information: {:?}", e);
                }
            }

            match response.response {
                ParseResponse::Block => {
                    warn!("blocking request");
                    self.set_http_response_body(0, body_size, &[]);
                }
                ParseResponse::Continue => (),
            }
        }

        Action::Continue
    }
}
