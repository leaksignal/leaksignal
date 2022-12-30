use std::{collections::HashMap, time::Duration};

use anyhow::{bail, Result};
use leakfinder::{
    BlockReason, BodyContext, FullHeader, Header, HttpParser, Match, ParseResponse, ParsedMatches,
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
    root::{block_state, DYN_ENVIRONMENT, LEAKFINDER_CONFIG},
    time::TIMESTAMP_PROVIDER,
    GIT_COMMIT,
};

const MATCH_PUSH_TIMEOUT: Duration = Duration::from_secs(5);

impl From<Header> for ProtoHeader {
    fn from(val: Header) -> Self {
        ProtoHeader {
            name: val.name,
            value: val.value,
        }
    }
}

impl From<Match> for ProtoMatch {
    fn from(val: Match) -> Self {
        ProtoMatch {
            category_name: val.category_name,
            global_start_position: val.global_start_position,
            global_length: val.global_length,
            matcher_path: val.matcher_path,
            matched_value: val.matched_value,
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

#[repr(i64)]
enum ListenerDirection {
    Unspecified = 0,
    Inbound = 1,
    Outbound = 2,
}

impl ListenerDirection {
    pub fn from_i64(v: i64) -> Option<Self> {
        match v {
            0 => Some(ListenerDirection::Unspecified),
            1 => Some(ListenerDirection::Inbound),
            2 => Some(ListenerDirection::Outbound),
            _ => None,
        }
    }
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

    fn get_property_int(&self, name: &str) -> Option<i64> {
        let raw = self.get_property_parse(name)?;
        if raw.len() != 8 {
            return None;
        }
        Some(i64::from_le_bytes(raw.try_into().unwrap()))
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
            metric.set_value(*us_taken);
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
        source: Option<&str>,
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
        if let Some(source) = source {
            headers.push(("x-ls-source", source));
        }
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

        self.collect_connection_info();

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

    fn collect_connection_info(&mut self) {
        if let Some(mtls) = self.get_property_bool("connection.mtls") {
            self.connection_info
                .insert("mtls".to_string(), mtls.to_string());
        }
        for property in STRING_CONNECTION_PROPERTIES {
            if let Some(value) = self.get_property_string(property) {
                self.connection_info.insert(property.to_string(), value);
            }
        }
        for property in INT_CONNECTION_PROPERTIES {
            if let Some(value) = self.get_property_int(property) {
                self.connection_info
                    .insert(property.to_string(), value.to_string());
            }
        }
    }

    fn check_token_blocked(&mut self, token: &str) -> bool {
        if let Some(reason) = block_state().is_ip_blocked(&**TIMESTAMP_PROVIDER, token) {
            warn!("blocking request by token {token} due to {reason}");
            if matches!(reason, BlockReason::Ratelimit) {
                self.early_response(429, vec![], None, Some(&*reason.to_string()));
            } else {
                self.early_response(403, vec![], None, Some(&*reason.to_string()));
            }
            true
        } else {
            false
        }
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

const INT_CONNECTION_PROPERTIES: &[&str] = &["listener_direction"];

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
                self.early_response(403, vec![], None, Some("blocked_ip"));
                return Action::Continue;
            }
            if let Some(reason) = block_state().is_ip_blocked(&**TIMESTAMP_PROVIDER, &ip) {
                warn!("blocking request by ip {ip} due to {reason}");
                self.parser().with_ip(ip);
                if matches!(reason, BlockReason::Ratelimit) {
                    self.early_response(429, vec![], None, Some(&*reason.to_string()));
                } else {
                    self.early_response(403, vec![], None, Some(&*reason.to_string()));
                }
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

        if let Some(local_service) =
            crate::service::local_service_name(|x| self.get_property_string(x))
        {
            let policy = self.parser_ref().policy();
            let service_policy = policy
                .services
                .iter()
                .find(|e| e.service_matched(&local_service));

            if let Some(service_policy) = service_policy {
                let direction = self
                    .get_property_int("listener_direction")
                    .and_then(ListenerDirection::from_i64);
                // envoy notion if inbound/outbound is reversed to ours
                if !matches!(direction, Some(ListenerDirection::Outbound)) {
                    let peer_service =
                        crate::service::outbound_peer_service_name(|x| self.get_property_string(x));
                    if !service_policy.inbound_allowed(peer_service.as_deref()) {
                        warn!(
                            "blocking request by service {} by policy",
                            peer_service.as_deref().unwrap_or("external")
                        );
                        self.early_response(403, vec![], None, Some("blocked_service"));
                        return Action::Continue;
                    }
                    if let Some(peer_service) = peer_service.as_deref() {
                        if let Some(reason) =
                            block_state().is_service_blocked(&**TIMESTAMP_PROVIDER, peer_service)
                        {
                            warn!("blocking request by service {peer_service} due to {reason}");
                            if matches!(reason, BlockReason::Ratelimit) {
                                self.early_response(429, vec![], None, Some(&*reason.to_string()));
                            } else {
                                self.early_response(403, vec![], None, Some(&*reason.to_string()));
                            }
                            return Action::Continue;
                        }
                    }
                }
            }
        }

        if let Some(token) = self.parser_ref().token().map(|x| x.to_string()) {
            if self.parser_ref().policy().blocked_tokens.contains(&*token) {
                warn!("blocking request by token {token}");
                self.early_response(403, vec![], None, Some("blocked_token"));
                return Action::Continue;
            }
            if self.check_token_blocked(&token) {
                return Action::Continue;
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
                    self.early_response(403, vec![], None, Some("blocked_match"));
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
            self.collect_connection_info();
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
        if let Some(token) = self.parser_ref().token().map(|x| x.to_string()) {
            if self.parser_ref().policy().blocked_tokens.contains(&*token) {
                warn!("blocking response by token {token}");
                self.early_response(403, vec![], None, Some("blocked_token"));
                return Action::Continue;
            }
            if self.check_token_blocked(&token) {
                return Action::Continue;
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
