use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Write as FmtWrite,
    io::Write,
    pin::Pin,
    str::FromStr,
    sync::Arc,
    task::Poll,
    time::{Duration, SystemTime},
};

use anyhow::{bail, Result};
use flate2::write::GzDecoder;
use futures::{task::waker, Future, FutureExt};
use leakpolicy::{ContentType, RegexWrapper};
use log::{error, warn};
use prost::Message;
use proxy_wasm::{
    hostcalls,
    traits::{Context, HttpContext},
    types::{Action, MetricType},
};
use rand::{thread_rng, Rng};
use sha2::{Digest, Sha256};

use crate::{
    config::{upstream, UpstreamConfigHandle, LEAKSIGNAL_SERVICE_NAME},
    metric::Metric,
    parsers::{grpc::parse_grpc, html::parse_html, json::parse_json, ParseResponse},
    pipe::{pipe, DummyWaker, PipeReader, PipeWriter},
    policy::{policy, PathPolicy, TokenExtractionConfig, TokenExtractionSite},
    proto::{Header, MatchDataRequest},
    root::DYN_ENVIRONMENT,
    GIT_COMMIT,
};

const MATCH_PUSH_TIMEOUT: Duration = Duration::from_secs(5);

impl Default for ContentEncoding {
    fn default() -> Self {
        ContentEncoding::None
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum ContentEncoding {
    Gzip,
    None,
    Unknown,
}

impl FromStr for ContentEncoding {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "gzip" => ContentEncoding::Gzip,
            _ => ContentEncoding::Unknown,
        })
    }
}

pub struct HttpResponseContext {
    //todo: remove when on_http_response_headers fixed (see internal comment)
    has_response_started: bool,
    data: Option<ResponseData>,
    content_encoding: ContentEncoding,
    decompressor: GzDecoder<Vec<u8>>,
    response_writer: Option<PipeWriter>,
    response_read_task: Option<Pin<Box<dyn Future<Output = Option<ResponseOutputData>>>>>,
    connection_info: HashMap<String, String>,
    request_headers: Vec<Header>,
    response_headers: Vec<Header>,
}

impl Default for HttpResponseContext {
    fn default() -> Self {
        Self {
            has_response_started: false,
            content_encoding: Default::default(),
            data: Some(Default::default()),
            response_writer: None,
            response_read_task: None,
            decompressor: GzDecoder::new(vec![]),
            connection_info: Default::default(),
            request_headers: vec![],
            response_headers: vec![],
        }
    }
}

#[derive(Default)]
struct ResponseData {
    request_start: u64,
    response_start: u64,
    response_body_start: u64,
    content_type: ContentType,
    content_encoding: ContentEncoding,
    path: String,
    token: Option<String>,
    policy: Option<PathPolicy>,
    ip: String,
}

struct ResponseOutputData {
    response: ParseResponse,
    packet: MatchDataRequest,
    upstream: Option<UpstreamConfigHandle>,
}

fn timestamp() -> u64 {
    hostcalls::get_current_time()
        .unwrap()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

async fn response_body_task(
    data: ResponseData,
    mut reader: PipeReader,
) -> Option<ResponseOutputData> {
    let policy = match policy() {
        Some(policy) => policy,
        None => {
            warn!("processing response, but no policy loaded");
            return None;
        }
    };
    // todo: cache this?
    let path_policy = data.policy.expect("missing policy");

    let mut matches = vec![];
    let category_performance_us: RefCell<HashMap<String, u64>> = RefCell::new(HashMap::new());

    let performance_measure = |name: &str, us: u64| {
        // we don't use entry here to avoid allocating name when unneeded
        let mut category_performance_us = category_performance_us.borrow_mut();
        match category_performance_us.get_mut(name) {
            Some(x) => {
                *x += us;
            }
            None => {
                category_performance_us.insert(name.to_string(), us);
            }
        }
    };

    let response = match data.content_type {
        ContentType::Html => {
            match parse_html(
                &*policy,
                &mut reader,
                &path_policy.configuration,
                &mut matches,
                performance_measure,
            )
            .await
            {
                Ok(x) => x,
                Err(e) => {
                    error!("failed to read html: {:?}", e);
                    return None;
                }
            }
        }
        ContentType::Json => {
            match parse_json(
                &*policy,
                &mut reader,
                &path_policy.configuration,
                &mut matches,
                performance_measure,
            )
            .await
            {
                Ok(x) => x,
                Err(e) => {
                    error!("failed to read json: {:?}", e);
                    return None;
                }
            }
        }
        ContentType::Grpc => {
            match parse_grpc(
                &*policy,
                &mut reader,
                &path_policy.configuration,
                &mut matches,
                performance_measure,
            )
            .await
            {
                Ok(x) => x,
                Err(e) => {
                    error!("failed to read grpc: {:?}", e);
                    return None;
                }
            }
        }
        // do no parsing here
        ContentType::Jpeg => ParseResponse::Continue, // parse_jpeg(&*body, &configuration),
        ContentType::Unknown => ParseResponse::Continue,
    };

    let body_size = reader.total_read() as u64;
    let body = if policy.body_collection_rate <= 0.0 || policy.max_body_collection_mb <= 0.0 {
        None
    } else {
        let chance: f64 = thread_rng().gen();
        if chance < policy.body_collection_rate {
            reader.fetch_full_content()
        } else {
            None
        }
    };

    let response_body_end = timestamp();

    let upstream = upstream();

    let mut match_counts: HashMap<&str, i64> = HashMap::new();
    for matching in &matches {
        *match_counts.entry(&*matching.category_name).or_default() += 1;
    }
    let policy_path = path_policy.policy_path;
    for (category_name, count) in match_counts {
        let metric = Metric::lookup_or_define(
            format!("ls.{policy_path}.{category_name}.count"),
            MetricType::Counter,
        );
        metric.increment(count);
    }
    for (category_name, us_taken) in &*category_performance_us.borrow() {
        let metric = Metric::lookup_or_define(
            format!("ls.{policy_path}.{category_name}.us_taken"),
            MetricType::Histogram,
        );
        metric.set_value(*us_taken as u64);
    }

    let packet = MatchDataRequest {
        api_key: upstream.as_ref().map(|x| x.api_key.clone()).flatten(),
        deployment_name: upstream
            .as_ref()
            .map(|x| x.deployment_name.clone())
            .unwrap_or_default(),
        policy_id: policy.policy_id().to_string(),
        highest_action_taken: crate::proto::Action::None as i32,
        time_request_start: data.request_start,
        time_response_start: data.response_start,
        time_response_body_start: data.response_body_start,
        time_response_body_end: response_body_end,
        time_response_parse_end: response_body_end,
        request_headers: Default::default(),
        response_headers: Default::default(),
        matches,
        body_size,
        body,
        policy_path,
        commit: GIT_COMMIT.to_string(),
        token: data.token.unwrap_or_default(),
        ip: data.ip,
        category_performance_us: category_performance_us.into_inner(),
        connection_info: Default::default(),
        environment: Default::default(),
    };

    Some(ResponseOutputData {
        response,
        packet,
        upstream,
    })
}

impl Context for HttpResponseContext {
    fn on_grpc_call_response(&mut self, _token_id: u32, status_code: u32, _response_size: usize) {
        if status_code != 0 {
            warn!("MatchData upload failed with status_code {status_code}");
        }
    }
}

impl HttpResponseContext {
    fn process_data(&mut self, body_size: usize) -> Result<Vec<u8>> {
        let body = match self.get_http_response_body(0, body_size) {
            Some(x) => x,
            None => {
                bail!("missing body for response");
            }
        };
        match self.content_encoding {
            ContentEncoding::Gzip => {
                self.decompressor.write_all(&body[..])?;
                Ok(self.decompressor.get_mut().drain(..).collect::<Vec<_>>())
            }
            ContentEncoding::None | ContentEncoding::Unknown => Ok(body),
        }
    }

    fn data(&mut self) -> &mut ResponseData {
        self.data.as_mut().unwrap()
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

fn extract_token_regex(value: &str, regex: Option<&RegexWrapper>, hash: bool) -> Option<String> {
    let value = match regex {
        Some(RegexWrapper(regex)) => {
            let captures = regex.captures(value).ok()??;
            if let Some(captured) = captures.get(1) {
                captured.as_str()
            } else {
                captures.get(0)?.as_str()
            }
        }
        None => value,
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
    fn on_http_request_headers(&mut self, _num_headers: usize, end_of_stream: bool) -> Action {
        if !end_of_stream {
            //TODO: this doesn't work for some reason
            // return Action::Pause;
        }
        let policy = match policy() {
            Some(policy) => policy,
            None => {
                warn!("processing request headers, but no policy loaded");
                return Action::Continue;
            }
        };

        if self.data().request_start == 0 {
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

            let path = self
                .get_http_request_header_bytes(":path")
                .map(|x| String::from_utf8_lossy(&x).into_owned())
                .unwrap_or_else(|| "/".to_string());
            let hostname = self
                .get_http_request_header_bytes(":authority")
                .map(|x| String::from_utf8_lossy(&x).into_owned())
                .unwrap_or_else(String::new);
            let data = self.data();
            data.ip = ip;
            data.request_start = timestamp();
            data.path = path;
            let full_path = format!("{}{}", hostname, data.path);
            data.policy = Some(policy.get_path_config(&*full_path));
        } else if self.data().policy.is_none() {
            return Action::Continue;
        }

        for (name, value) in self.get_http_request_headers_bytes() {
            let value = match String::from_utf8(value) {
                Ok(x) => x,
                Err(_) => continue,
            };

            let data = self.data();

            if let Some(policy) = &data.policy {
                match policy.token_extractor.as_deref() {
                    Some(TokenExtractionConfig {
                        location: TokenExtractionSite::Request,
                        header,
                        regex,
                        hash,
                    }) if &name == header => {
                        data.token = extract_token_regex(&*value, regex.as_ref(), *hash);
                    }
                    Some(TokenExtractionConfig {
                        location: TokenExtractionSite::RequestCookie,
                        header,
                        regex,
                        hash,
                    }) if name == "cookie" => {
                        for value in value.split("; ") {
                            let (name, value) = match value.split_once('=') {
                                Some(x) => x,
                                None => continue,
                            };
                            if name == header {
                                data.token = extract_token_regex(value, regex.as_ref(), *hash);
                                break;
                            }
                        }
                    }
                    _ => (),
                }
            }

            let value = if policy.collected_request_headers.contains(&name) {
                Some(value)
            } else {
                None
            };

            self.request_headers.push(Header { name, value });
        }
        Action::Continue
    }

    fn on_http_response_headers(&mut self, _: usize, end_of_stream: bool) -> Action {
        if !end_of_stream {
            //TODO: this doesn't work for some reason
            // return Action::Pause;
        }
        let policy = match policy() {
            Some(policy) => policy,
            None => {
                return Action::Continue;
            }
        };

        if !self.has_response_started {
            self.has_response_started = true;
            self.data.as_mut().unwrap().response_start = timestamp();
        }
        self.set_http_response_header("content-length", None);
        for (name, value) in self.get_http_response_headers_bytes() {
            let value = String::from_utf8_lossy(&value).into_owned();

            let data = self.data();
            if let Some(policy) = &data.policy {
                match policy.token_extractor.as_deref() {
                    Some(TokenExtractionConfig {
                        location: TokenExtractionSite::Response,
                        header,
                        regex,
                        hash,
                    }) if &name == header => {
                        data.token = extract_token_regex(&*value, regex.as_ref(), *hash);
                    }
                    _ => (),
                }
            }

            let value = if policy.collected_response_headers.contains(&name) {
                Some(value)
            } else {
                None
            };

            self.response_headers.push(Header { name, value });
        }

        //todo: we might need to cover multiple content-type headers here
        let content_type = self.get_http_response_header_bytes("content-type");
        self.data.as_mut().unwrap().content_type =
            match content_type.map(|x| String::from_utf8_lossy(&x).into_owned()) {
                Some(value) => value
                    .parse()
                    .expect("content-type parse failed (impossible)"),
                None => ContentType::Unknown,
            };
        let content_encoding = self.get_http_response_header_bytes("content-encoding");
        self.data.as_mut().unwrap().content_encoding =
            match content_encoding.map(|x| String::from_utf8_lossy(&x).into_owned()) {
                Some(value) => value
                    .trim()
                    .parse()
                    .expect("content-encoding parse failed (impossible)"),
                None => ContentEncoding::None,
            };
        self.content_encoding = self.data.as_ref().unwrap().content_encoding;
        Action::Continue
    }

    fn on_http_response_trailers(&mut self, _: usize) -> Action {
        let policy = match policy() {
            Some(policy) => policy,
            None => {
                return Action::Continue;
            }
        };

        for (name, value) in self.get_http_response_trailers_bytes() {
            let value = String::from_utf8_lossy(&value).into_owned();

            let value = if policy.collected_response_headers.contains(&name) {
                Some(value)
            } else {
                None
            };

            self.response_headers.push(Header { name, value });
        }

        Action::Continue
    }

    fn on_http_response_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        if let Some(data) = &mut self.data {
            if data.policy.is_none() {
                return Action::Continue;
            }

            data.response_body_start = timestamp();
            let max_persistence = policy()
                .map(|x| (x.max_body_collection_mb * 1024.0 * 1024.0) as usize)
                .unwrap_or_default();
            let (reader, writer) = pipe(max_persistence);
            self.response_writer = Some(writer);
            self.response_read_task = Some(Box::pin(response_body_task(
                self.data.take().unwrap(),
                reader,
            )));
        }
        // cleared when we fail to do anything (i.e. no policy).
        // when there is more chunks of that original response, continue skipping here.
        if self.response_read_task.is_none() {
            return Action::Continue;
        }

        if body_size > 0 {
            let body = match self.process_data(body_size) {
                Err(e) => {
                    error!("failed to read body: {:?}", e);
                    return Action::Continue;
                }
                Ok(x) => x,
            };
            if !body.is_empty() {
                self.response_writer
                    .as_mut()
                    .expect("receives data after end_of_stream")
                    .append(body);
            }
        }
        if end_of_stream {
            if matches!(self.content_encoding, ContentEncoding::Gzip) {
                let body = match std::mem::replace(&mut self.decompressor, GzDecoder::new(vec![]))
                    .finish()
                {
                    Err(e) => {
                        error!("failed to read body: {:?}", e);
                        return Action::Continue;
                    }
                    Ok(x) => x,
                };
                if !body.is_empty() {
                    self.response_writer
                        .as_mut()
                        .expect("receives data after end_of_stream")
                        .append(body);
                }
            }
            self.response_writer.take().unwrap();
        } else if body_size == 0 {
            return Action::Continue;
        }

        let waker = waker(Arc::new(DummyWaker));
        let mut context = std::task::Context::from_waker(&waker);
        match self
            .response_read_task
            .as_mut()
            .unwrap()
            .poll_unpin(&mut context)
        {
            Poll::Ready(None) => {
                self.response_read_task.take();
                Action::Continue
            }
            Poll::Ready(Some(data)) => {
                self.response_read_task.take();
                let mut packet = data.packet;
                packet.connection_info = std::mem::take(&mut self.connection_info);
                packet.request_headers = std::mem::take(&mut self.request_headers);
                packet.response_headers = std::mem::take(&mut self.response_headers);
                packet.environment = DYN_ENVIRONMENT
                    .load()
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect();
                if let Some(upstream) = data.upstream {
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
                match data.response {
                    ParseResponse::Block => {
                        warn!("blocking request for '{}'", packet.policy_path);
                        self.set_http_response_body(0, body_size, &[]);
                        Action::Continue
                    }
                    ParseResponse::Continue => Action::Continue,
                }
            }
            Poll::Pending => Action::Continue,
        }
    }
}
