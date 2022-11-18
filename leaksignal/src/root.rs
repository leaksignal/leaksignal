use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, SystemTime},
};

use arc_swap::ArcSwap;
use indexmap::IndexMap;
use leakfinder::{PolicyHolder, TimestampProvider};
use leakpolicy::{parse_policy, Policy};
use log::{debug, error, warn};
use prost::Message;
use proxy_wasm::{
    traits::{Context, HttpContext, RootContext},
    types::{ContextType, Status},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    config::{
        create_service_definition, update_upstream, upstream, Config, Mode, UpstreamConfig,
        LEAKSIGNAL_SERVICE_NAME,
    },
    env::ENVIRONMENT,
    http_response::HttpResponseContext,
    proto::{PingMessage, UpdatePolicyRequest, UpdatePolicyResponse},
    time::TIMESTAMP_PROVIDER,
    GIT_COMMIT,
};

const POLICY_SHARED_QUEUE_PREFIX: &str = "leaksignal_queue_";
const LOCAL_COLLECTOR_VM: &str = "leaksignal_service";
const PROXY_VM: &str = "leaksignal_proxy";

lazy_static::lazy_static! {
    pub static ref DYN_ENVIRONMENT: ArcSwap<IndexMap<String, String>> = ArcSwap::new(Arc::new(ENVIRONMENT.clone()));
    pub static ref POLICY: Arc<PolicyHolder> = Default::default();
    pub static ref LEAKFINDER_CONFIG: leakfinder::Config = leakfinder::Config {
        policy: POLICY.clone(),
        timestamp_source: TIMESTAMP_PROVIDER.clone(),
    };
}

#[derive(Serialize)]
pub enum FilterInboundMessageRef<'a> {
    PolicyUpdate {
        id: &'a str,
        policy: &'a Policy,
        environment_variables: &'a IndexMap<String, String>,
    },
    UpstreamUpdate(Option<&'a UpstreamConfig>),
}

#[derive(Deserialize)]
pub enum FilterInboundMessage {
    PolicyUpdate {
        id: String,
        policy: Policy,
        environment_variables: IndexMap<String, String>,
    },
    UpstreamUpdate(Option<UpstreamConfig>),
}

pub enum RootContextData {
    Unknown,
    DirectFilter,
    Collector {
        worker_register_queue: u32,
        policy_update_queues: Vec<u32>,
    },
    Filter {
        policy_update_queue: u32,
        uuid: Uuid,
        worker_register_queue: Option<u32>,
    },
}

impl RootContextData {
    fn filter_worker_register_queue(&mut self) -> &mut Option<u32> {
        match self {
            RootContextData::Filter {
                worker_register_queue,
                ..
            } => worker_register_queue,
            _ => panic!("tried to get worker_register_queue on non-filter"),
        }
    }

    fn filter_uuid(&self) -> Uuid {
        match self {
            RootContextData::Filter { uuid, .. } => *uuid,
            _ => panic!("tried to get uuid on non-filter"),
        }
    }
}

impl Default for RootContextData {
    fn default() -> Self {
        RootContextData::Unknown
    }
}

#[derive(Default)]
pub struct EnvoyRootContext {
    policy_stream_id: Option<u32>,
    data: RootContextData,
    last_ping_sent: Option<Duration>,
    // (timestamp, token_id)
    last_ping_data: Option<(u64, u32)>,
}

const PING_INTERVAL: Duration = Duration::from_secs(300);
const PING_TIMEOUT: Duration = Duration::from_secs(20);

impl Context for EnvoyRootContext {
    fn on_grpc_stream_message(&mut self, token_id: u32, message_size: usize) {
        if Some(token_id) != self.policy_stream_id {
            return;
        }
        let body = self
            .get_grpc_stream_message(0, message_size)
            .expect("missing grpc stream body");

        let response = match UpdatePolicyResponse::decode(&body[..]) {
            Ok(x) => x,
            Err(e) => {
                error!("failed to decode policy response: {:?}", e);
                return;
            }
        };
        warn!("received new leaksignal policy: {}", response.policy_id);

        let policy: Policy = match parse_policy(&*response.policy) {
            Ok(x) => x,
            Err(e) => {
                error!("failed to parse new policy: {:?}", e);
                return;
            }
        };

        self.do_policy_update(response.policy_id.clone(), policy);
    }

    fn on_grpc_stream_close(&mut self, token_id: u32, status_code: u32) {
        if Some(token_id) != self.policy_stream_id {
            return;
        }
        warn!(
            "policy update stream closed, restarting within 15 seconds (error code {})",
            status_code
        );
        self.policy_stream_id = None;
    }

    fn on_grpc_call_response(&mut self, token_id: u32, status_code: u32, response_size: usize) {
        if let Some((timestamp, ping_token_id)) = self.last_ping_data {
            if ping_token_id == token_id {
                self.last_ping_data = None;
                if status_code != 0 {
                    warn!("leaksignal infra ping failed (error code {})", status_code);
                    return;
                }
                let body = self
                    .get_grpc_call_response_body(0, response_size)
                    .expect("missing grpc ping response body");

                let response = match PingMessage::decode(&body[..]) {
                    Ok(x) => x,
                    Err(e) => {
                        error!("failed to decode ping response: {:?}", e);
                        return;
                    }
                };

                if response.timestamp != timestamp {
                    warn!(
                        "ping response timestamp did not match request timestamp: {} != {}",
                        response.timestamp, timestamp
                    );
                }
            }
        }
    }
}

fn policy_update<'a>(id: &'a str, policy: &'a Policy) -> FilterInboundMessageRef<'a> {
    FilterInboundMessageRef::PolicyUpdate {
        id,
        policy,
        environment_variables: &*ENVIRONMENT,
    }
}

impl EnvoyRootContext {
    fn do_policy_update(&mut self, policy_id: impl Into<String>, policy: Policy) {
        let policy_id: String = policy_id.into();
        if Config::get().mode() == Mode::LocalCollector {
            self.broadcast_to_workers(policy_update(&*policy_id, &policy));
        }

        POLICY.update_policy(policy_id, policy);
    }

    fn broadcast_to_workers(&mut self, message: FilterInboundMessageRef) {
        assert_eq!(Config::get().mode(), Mode::LocalCollector);

        let mut bad_stream_index = vec![];
        if let RootContextData::Collector {
            policy_update_queues,
            ..
        } = &self.data
        {
            let message =
                serde_json::to_string(&message).expect("failed to serialize PolicyUpdate message");
            for (i, queue) in policy_update_queues.iter().enumerate() {
                match self.enqueue_shared_queue(*queue, Some(message.as_bytes())) {
                    Ok(()) => (),
                    Err(Status::NotFound) => {
                        bad_stream_index.push(i);
                    }
                    Err(e) => {
                        error!("failed to send policy to worker thread: {:?}", e);
                    }
                }
            }
        }
        if let RootContextData::Collector {
            policy_update_queues,
            ..
        } = &mut self.data
        {
            for index in bad_stream_index.into_iter().rev() {
                policy_update_queues.remove(index);
            }
        }
    }

    fn try_start_update_policy(&mut self) {
        let config = Config::get();
        let upstream = upstream();
        match config.mode() {
            Mode::DirectFilter | Mode::LocalCollector
                if self.policy_stream_id.is_none() && upstream.is_some() =>
            {
                // set tick period to 15 seconds always
                self.set_tick_period(Duration::from_secs(15));

                let upstream = upstream.unwrap();
                let policy_request = if config.local_policy.is_none() {
                    warn!(
                        "loading policy for '{}' deployment",
                        upstream.deployment_name
                    );
                    let request = UpdatePolicyRequest {
                        api_key: upstream.api_key.clone(),
                        deployment_name: upstream.deployment_name.clone(),
                        commit: GIT_COMMIT.to_string(),
                    };
                    let request = request.encode_to_vec();

                    Some(request)
                } else {
                    None
                };

                self.policy_stream_id = match self.open_grpc_stream(
                    unsafe { std::str::from_utf8_unchecked(&upstream.service_definition[..]) },
                    LEAKSIGNAL_SERVICE_NAME,
                    "UpdatePolicy",
                    vec![],
                ) {
                    Ok(policy_stream_id) => {
                        if let Some(policy_request) = policy_request {
                            self.send_grpc_stream_message(
                                policy_stream_id,
                                Some(&policy_request[..]),
                                false,
                            );
                        }
                        Some(policy_stream_id)
                    }
                    Err(e) => {
                        error!("failed to start policy configuration: {:?}", e);
                        None
                    }
                };
                self.last_ping_sent = None;
            }
            Mode::Filter if self.data.filter_worker_register_queue().is_none() => {
                *self.data.filter_worker_register_queue() = self.resolve_shared_queue(
                    LOCAL_COLLECTOR_VM,
                    &*format!("{}{}", POLICY_SHARED_QUEUE_PREFIX, config.group),
                );
                if let Some(worker_register_queue) = *self.data.filter_worker_register_queue() {
                    self.enqueue_shared_queue(
                        worker_register_queue,
                        Some(self.data.filter_uuid().as_bytes()),
                    )
                    .unwrap();
                }
            }
            _ => (),
        }
    }
}

static CONFIGURED: AtomicBool = AtomicBool::new(false);

impl RootContext for EnvoyRootContext {
    fn on_configure(&mut self, plugin_configuration_size: usize) -> bool {
        let parsed_config: Config = match self.get_plugin_configuration() {
            Some(config) => serde_yaml::from_slice(&config[..plugin_configuration_size])
                .expect("failed to parse proxy_wasm configuration"),
            None => Config::default(),
        };
        if let Some(previous_config) = Config::try_get() {
            if previous_config.mode() != parsed_config.mode() {
                error!("cannot change mode of LeakSignal module without an Envoy restart, config change ignored.");
                return true;
            }
            if previous_config.group != parsed_config.group {
                error!("cannot change group of LeakSignal module without an Envoy restart, config change ignored.");
                return true;
            }
        }
        parsed_config.set();
        debug!("leaksignal config reloaded");
        let config = Config::get();
        let old_upstream = upstream();
        if let (Some(upstream_cluster), Some(deployment_name)) =
            (&config.upstream_cluster, &config.deployment_name)
        {
            update_upstream(Some(UpstreamConfig {
                service_definition: create_service_definition(upstream_cluster),
                deployment_name: deployment_name.clone(),
                api_key: config.api_key.clone(),
            }));
        }

        if let Some(local_policy) = &config.local_policy {
            self.do_policy_update("local", local_policy.clone());
        }

        if config.upstream_cluster.is_some() != config.deployment_name.is_some() {
            error!("must specify both `upstream_cluster` and `deployment_name` in config, or neither. values are ignored.");
        }

        if CONFIGURED.fetch_or(true, Ordering::Relaxed) {
            // restart upstream connection if upstream changed
            let new_upstream = upstream();
            if new_upstream.as_deref() != old_upstream.as_deref() {
                if matches!(config.mode(), Mode::LocalCollector | Mode::DirectFilter) {
                    self.policy_stream_id = None;
                    self.try_start_update_policy();
                }
                if config.mode() == Mode::LocalCollector && new_upstream.is_some() {
                    self.broadcast_to_workers(FilterInboundMessageRef::UpstreamUpdate(
                        new_upstream.as_deref(),
                    ));
                }
            }
        } else {
            match config.mode() {
                Mode::LocalCollector => {
                    self.data = RootContextData::Collector {
                        worker_register_queue: self.register_shared_queue(&*format!(
                            "{}{}",
                            POLICY_SHARED_QUEUE_PREFIX, config.group
                        )),
                        policy_update_queues: vec![],
                    };
                }
                Mode::Filter => {
                    let uuid = Uuid::new_v4();
                    self.data = RootContextData::Filter {
                        policy_update_queue: self.register_shared_queue(&*format!(
                            "{}{}_{}",
                            POLICY_SHARED_QUEUE_PREFIX, uuid, config.group
                        )),
                        uuid,
                        worker_register_queue: None,
                    };
                }
                Mode::DirectFilter => {
                    self.data = RootContextData::DirectFilter;
                }
            }

            if matches!(config.mode(), Mode::DirectFilter | Mode::LocalCollector) {
                // we want to wait one second before we try to connect to infrastructure
                // this is to avoid envoy having not resolved DNS yet
                self.set_tick_period(Duration::from_secs(1));
            } else {
                // Mode::Filter needs to call this only once
                self.try_start_update_policy();
            }
        }

        true
    }

    fn on_tick(&mut self) {
        let config = Config::get();
        assert!(matches!(
            config.mode(),
            Mode::DirectFilter | Mode::LocalCollector
        ));
        drop(config);
        let upstream = match upstream() {
            Some(x) => x,
            None => return,
        };

        let now = TIMESTAMP_PROVIDER.elapsed();

        if let (Some((timestamp, token)), Some(last_ping_sent)) =
            (self.last_ping_data, self.last_ping_sent)
        {
            if now.checked_sub(last_ping_sent).unwrap_or_default() > PING_TIMEOUT {
                warn!("ping timed out at proxy layer (envoy error?) timestamp: {timestamp} ping token: {token}");
                self.policy_stream_id = None;
                self.last_ping_data = None;
            }
        }

        if self.last_ping_sent.is_none()
            || now
                .checked_sub(self.last_ping_sent.unwrap())
                .unwrap_or_default()
                > PING_INTERVAL
        {
            self.last_ping_sent = Some(now);

            let timestamp = self
                .get_current_time()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64;

            let ping = PingMessage { timestamp };

            match self.dispatch_grpc_call(
                unsafe { std::str::from_utf8_unchecked(&upstream.service_definition[..]) },
                LEAKSIGNAL_SERVICE_NAME,
                "Ping",
                vec![],
                Some(&ping.encode_to_vec()[..]),
                PING_TIMEOUT,
            ) {
                Err(e) => {
                    error!("failed to send upstream ping: {:?}", e);
                }
                Ok(token) => {
                    self.last_ping_data = Some((timestamp, token));
                }
            }
        }

        self.try_start_update_policy();
    }

    fn get_type(&self) -> Option<ContextType> {
        let config = Config::get();

        match config.mode() {
            Mode::DirectFilter | Mode::Filter => Some(ContextType::HttpContext),
            Mode::LocalCollector => None,
        }
    }

    fn on_queue_ready(&mut self, queue_id: u32) {
        let config = Config::get();

        match &self.data {
            RootContextData::Unknown | RootContextData::DirectFilter => (),
            RootContextData::Collector {
                worker_register_queue,
                ..
            } => {
                if queue_id == *worker_register_queue {
                    let value = match self.dequeue_shared_queue(queue_id).unwrap() {
                        None => {
                            warn!("empty worker register queue?");
                            return;
                        }
                        Some(value) => value,
                    };
                    if value.len() != 16 {
                        error!("invalid uuid in register queue");
                        return;
                    }
                    let uuid = Uuid::from_bytes((&value[..]).try_into().unwrap());
                    let queue = match self.resolve_shared_queue(
                        PROXY_VM,
                        &*format!("{}{}_{}", POLICY_SHARED_QUEUE_PREFIX, uuid, config.group),
                    ) {
                        Some(x) => x,
                        None => {
                            error!("missing registered policy queue");
                            return;
                        }
                    };
                    if let Some(policy) = POLICY.policy() {
                        let message = policy_update(policy.policy_id(), &*policy);
                        let message = serde_json::to_string(&message)
                            .expect("failed to serialize PolicyUpdate message");
                        if let Err(e) = self.enqueue_shared_queue(queue, Some(message.as_bytes())) {
                            error!("failed to send initial policy to worker: {:?}", e);
                        }
                    }
                    let upstream = upstream();
                    let message = FilterInboundMessageRef::UpstreamUpdate(upstream.as_deref());
                    let message = serde_json::to_string(&message)
                        .expect("failed to serialize UpstreamUpdate message");
                    if let Err(e) = self.enqueue_shared_queue(queue, Some(message.as_bytes())) {
                        error!("failed to send initial upstream to worker: {:?}", e);
                    }

                    match &mut self.data {
                        RootContextData::Collector {
                            policy_update_queues,
                            ..
                        } => {
                            policy_update_queues.push(queue);
                        }
                        _ => unreachable!(),
                    }
                }
            }
            RootContextData::Filter {
                policy_update_queue,
                ..
            } => {
                if *policy_update_queue == queue_id {
                    let value = match self.dequeue_shared_queue(queue_id).unwrap() {
                        None => {
                            warn!("empty policy queue?");
                            return;
                        }
                        Some(value) => value,
                    };
                    let value: FilterInboundMessage = match serde_json::from_slice(&*value) {
                        Ok(x) => x,
                        Err(e) => {
                            error!("failed to deserialize message from local collector, version mismatch? (restarting envoy may fix this): {e:?}");
                            return;
                        }
                    };
                    match value {
                        FilterInboundMessage::PolicyUpdate {
                            id,
                            policy,
                            environment_variables,
                        } => {
                            POLICY.update_policy(id, policy);
                            DYN_ENVIRONMENT.store(Arc::new(environment_variables));
                        }
                        FilterInboundMessage::UpstreamUpdate(upstream) => {
                            update_upstream(upstream);
                        }
                    }
                }
            }
        }
    }

    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        let config = Config::get();

        match config.mode() {
            Mode::DirectFilter | Mode::Filter => Some(Box::new(HttpResponseContext::default())),
            Mode::LocalCollector => None,
        }
    }
}
