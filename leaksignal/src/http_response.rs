use std::{collections::HashMap, time::Duration};

use anyhow::{bail, Result};
use leakfinder::{
    BodyContext, FullHeader, Header, HttpParser, Match, ParseResponse, ParsedMatches,
    TimestampProvider,
};
use log::{error, warn};
use prost::Message;
use proxy_wasm::{
    traits::{Context, HttpContext},
    types::{Action, MetricType},
};

use crate::{
    config::{upstream, LEAKSIGNAL_SERVICE_NAME},
    metric::Metric,
    proto::{Component, Header as ProtoHeader, Match as ProtoMatch, MatchDataRequest},
    root::{DYN_ENVIRONMENT, LEAKFINDER_CONFIG},
    time::TIMESTAMP_PROVIDER,
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
    response_started: bool,
    request_body: Option<BodyContext>,
    response_body: Option<BodyContext>,
}

impl Default for HttpResponseContext {
    fn default() -> Self {
        Self {
            parser: LEAKFINDER_CONFIG.http_parser(),
            connection_info: Default::default(),
            request_started: false,
            response_started: false,
            request_body: None,
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
    fn receive_response_body_chunk(&mut self, body_size: usize) -> Result<Vec<u8>> {
        let body = match self.get_http_response_body(0, body_size) {
            Some(x) => x,
            None => {
                bail!("missing body for response");
            }
        };
        Ok(body)
    }

    fn receive_request_body_chunk(&mut self, body_size: usize) -> Result<Vec<u8>> {
        let body = match self.get_http_request_body(0, body_size) {
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

    fn parser_ref(&self) -> &HttpParser<'static> {
        self.parser.as_ref().unwrap()
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

    fn finish_internal(&mut self) {
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
                time_request_start: 0,
                time_response_start: 0,
                time_response_body_start: 0,
                time_response_body_end: 0,
                request_headers: Default::default(),
                response_headers: Default::default(),
                matches: Default::default(),
                body_size: Default::default(),
                body: Default::default(),
                policy_path: output.policy_path,
                commit: GIT_COMMIT.to_string(),
                token: output.token,
                ip: output.ip,
                category_performance_us: Default::default(),
                connection_info: std::mem::take(&mut self.connection_info),
                environment: DYN_ENVIRONMENT
                    .load()
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect(),
                request: Some(Component {
                    time_header_start: output.time_request_start,
                    time_body_start: output.request.time_parse_start,
                    time_body_end: output.request.time_parse_end,
                    headers: output.request_headers.into_iter().map(Into::into).collect(),
                    matches: output.request.matches.into_iter().map(Into::into).collect(),
                    body_size: output.request.body_size,
                    body: output.request.body,
                    category_performance_us: output.request.category_performance_us,
                }),
                response: Some(Component {
                    time_header_start: output.time_response_start,
                    time_body_start: output.response.time_parse_start,
                    time_body_end: output.response.time_parse_end,
                    headers: output
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
                    category_performance_us: output.response.category_performance_us,
                }),
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
    }

    fn early_response(
        &mut self,
        status_code: u32,
        headers: Vec<(&str, &str)>,
        body: Option<&[u8]>,
    ) {
        if let Some(request) = self.request_body.take() {
            let request = match request.end_stream() {
                Err(e) => {
                    error!("failed to decode body at end of stream: {e:?}");
                    return;
                }
                Ok(x) => x,
            };
            self.parser().finish_request_stream(request.matches);
        }

        let request_id = self.get_property_string("request.id").unwrap_or_default();
        let mut headers: Vec<_> = headers.into_iter().collect();
        headers.push(("x-ls-request-id", &*request_id));
        headers.push(("x-source", "leaksignal"));
        for (name, value) in &headers {
            self.parser().with_response_headers([FullHeader {
                name: name.to_string(),
                value: value.to_string(),
            }]);
        }
        self.parser().with_response_headers([FullHeader {
            name: ":status".to_string(),
            value: status_code.to_string(),
        }]);

        let now = TIMESTAMP_PROVIDER.epoch_ns();

        self.send_http_response(status_code, headers, body);
        self.parser().finish_response_stream(ParsedMatches {
            matches: vec![],
            body_size: 0,
            body: None,
            category_performance_us: Default::default(),
            time_parse_start: now,
            time_parse_end: now,
        });
        self.finish_internal();
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
            if self.parser().policy().blocked_ips.contains(&ip) {
                warn!("blocking request by ip {ip}");
                self.parser().with_ip(ip);
                self.early_response(403, vec![], None);
                return Action::Continue;
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
        if let Some(token) = self.parser_ref().token() {
            if self.parser_ref().policy().blocked_tokens.contains(token) {
                warn!("blocking request by token {token}");
                self.early_response(403, vec![], None);
            }
        }

        Action::Continue
    }

    fn on_http_request_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        if self.parser.is_none() {
            return Action::Continue;
        }

        if self.request_body.is_none() {
            self.request_body = self.parser().with_request_stream();
            if self.request_body.is_none() {
                warn!("unable to create request stream, were we missing :path or :authority?");
                return Action::Continue;
            }
        }

        if body_size > 0 {
            let parse_request = match self
                .receive_request_body_chunk(body_size)
                .and_then(|body| self.request_body.as_mut().unwrap().receive_chunk(body))
            {
                Err(e) => {
                    error!("failed to decode request body: {e:?}");
                    return Action::Continue;
                }
                Ok(x) => x,
            };

            match parse_request {
                ParseResponse::Block => {
                    warn!("blocking request");
                    self.early_response(403, vec![], None);
                    return Action::Continue;
                }
                ParseResponse::Continue => (),
            }
        }

        if end_of_stream {
            let request = match self.request_body.take().unwrap().end_stream() {
                Err(e) => {
                    error!("failed to decode body at end of stream: {e:?}");
                    return Action::Continue;
                }
                Ok(x) => x,
            };
            self.parser().finish_request_stream(request.matches);

            match request.response {
                ParseResponse::Block => {
                    warn!("blocking request");
                    self.send_http_response(403, vec![], None);
                }
                ParseResponse::Continue => (),
            }
        }

        Action::Continue
    }

    fn on_http_response_headers(&mut self, _: usize, _end_of_stream: bool) -> Action {
        if self.parser.is_none() {
            return Action::Continue;
        }

        if !self.response_started {
            self.response_started = true;
            if let Some(mtls) = self.get_property_bool("connection.mtls") {
                self.connection_info
                    .insert("mtls".to_string(), mtls.to_string());
            }
            for property in STRING_CONNECTION_PROPERTIES {
                if let Some(value) = self.get_property_string(property) {
                    self.connection_info.insert(property.to_string(), value);
                }
            }
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
        if let Some(token) = self.parser_ref().token() {
            if self.parser_ref().policy().blocked_tokens.contains(token) {
                warn!("blocking response by token {token}");
                self.early_response(403, vec![], None);
            }
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
                .receive_response_body_chunk(body_size)
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
                    warn!("blocking response");
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
            self.finish_internal();

            match response.response {
                ParseResponse::Block => {
                    warn!("blocking response");
                    self.set_http_response_body(0, body_size, &[]);
                }
                ParseResponse::Continue => (),
            }
        }

        Action::Continue
    }
}

impl Drop for HttpResponseContext {
    fn drop(&mut self) {
        let response = match self.response_body.take().map(|x| x.end_stream()) {
            None => return,
            Some(Err(e)) => {
                error!("failed to decode body at end of stream: {e:?}");
                return;
            }
            Some(Ok(x)) => x,
        };
        // sometimes end_of_stream is never sent for responses in GRPC?
        self.parser().finish_response_stream(response.matches);
        self.finish_internal();
    }
}
