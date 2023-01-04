use std::{borrow::Cow, ops::Deref, sync::Arc};

use arc_swap::{ArcSwap, Guard};
use leakpolicy::Policy;
use prost::Message;
use proxy_wasm::hostcalls;
use serde::{Deserialize, Serialize};

mod grpc_service {
    include!(concat!(env!("OUT_DIR"), "/envoy.config.core.v3.rs"));
}

use self::grpc_service::grpc_service::{EnvoyGrpc, TargetSpecifier};

fn default_group() -> String {
    "default".to_string()
}

fn default_enable_metrics() -> bool {
    true
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Config {
    /// Specifies a specific deployment group within the locally running Envoy
    /// Can be used to run multiple distinct Leaksignal deployments
    /// Defaults to 'default'
    /// This cannot be changed without restarting Envoy.
    #[serde(default = "default_group")]
    pub group: String,
    /// Name of Envoy cluster that points to upstream/leaksignal command
    /// Can be omitted on filters in favor of the bootstrap plugin (LocalCollector) forwarding it
    pub upstream_cluster: Option<String>,
    /// API Key to send to upstream/leaksignal command
    /// Can be omitted on filters in favor of the bootstrap plugin (LocalCollector) forwarding it
    pub api_key: Option<String>,
    /// Deployment name field for upstream/leaksignal command
    /// Can be omitted on filters in favor of the bootstrap plugin (LocalCollector) forwarding it
    pub deployment_name: Option<String>,
    /// Running mode -- can be inferred from plugin VM id. If the plugin VM id contains `service`, it is assumed to be a bootstrap plugin, and therefor a LocalCollector
    /// Otherwise, it is assumed to be a Filter.
    /// This cannot be changed without restarting Envoy.
    pub mode: Option<Mode>,
    /// If set, a policy is parsed and used instead of requesting a policy from upstream/leaksignal command
    /// Alerts and other upstream functionality will, of course, not work.
    pub local_policy: Option<Policy>,
    /// If true, metrics are pushed upstream via Envoy's metrics collection system
    /// Defaults to true.
    #[serde(default = "default_enable_metrics")]
    pub enable_metrics: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            group: default_group(),
            upstream_cluster: None,
            api_key: None,
            deployment_name: None,
            mode: None,
            local_policy: None,
            enable_metrics: default_enable_metrics(),
        }
    }
}

pub struct ConfigRef(Guard<Arc<Option<Config>>>);

impl Deref for ConfigRef {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        (**self.0).as_ref().unwrap()
    }
}

lazy_static::lazy_static! {
    static ref IS_BOOTSTRAP: bool = {
        let plugin_vm_id = hostcalls::get_property(vec!["plugin_vm_id"]).expect("couldn't fetch plugin_vm_id");
        let plugin_vm_id = if let Some(x) = &plugin_vm_id {
            String::from_utf8_lossy(x)
        } else {
            Cow::Borrowed("")
        };
        plugin_vm_id.contains("service")
    };
}

impl Config {
    pub fn mode(&self) -> Mode {
        if *IS_BOOTSTRAP {
            self.mode.unwrap_or(Mode::LocalCollector)
        } else {
            self.mode.unwrap_or(Mode::Filter)
        }
    }

    pub fn try_get() -> Option<ConfigRef> {
        let config = CONFIG.load();
        config.is_some().then_some(ConfigRef(config))
    }

    pub fn get() -> ConfigRef {
        let config = CONFIG.load();
        config
            .is_some()
            .then_some(ConfigRef(config))
            .expect("referenced config with none loaded")
    }

    pub fn set(self) {
        CONFIG.store(Arc::new(Some(self)))
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    DirectFilter,
    Filter,
    LocalCollector,
}

lazy_static::lazy_static! {
    static ref CONFIG: ArcSwap<Option<Config>> = ArcSwap::new(Arc::new(None));
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct UpstreamConfig {
    #[serde(with = "hex::serde")]
    pub service_definition: Vec<u8>,
    pub deployment_name: String,
    pub api_key: Option<String>,
}

pub const LEAKSIGNAL_SERVICE_NAME: &str = "leaksignal.Leaksignal";

pub fn create_service_definition(upstream_cluster: &str) -> Vec<u8> {
    let service = grpc_service::GrpcService {
        target_specifier: Some(TargetSpecifier::EnvoyGrpc(EnvoyGrpc {
            cluster_name: upstream_cluster.to_string(),
            authority: upstream_cluster.to_string(),
        })),
        ..Default::default()
    };
    service.encode_to_vec()
}

pub fn update_upstream(config: Option<UpstreamConfig>) {
    LEAKSIGNAL_UPSTREAM.store(Arc::new(config));
}

pub struct UpstreamConfigHandle(Guard<Arc<Option<UpstreamConfig>>>);

impl Deref for UpstreamConfigHandle {
    type Target = UpstreamConfig;

    fn deref(&self) -> &Self::Target {
        (**self.0).as_ref().unwrap()
    }
}

pub fn upstream() -> Option<UpstreamConfigHandle> {
    let loaded = LEAKSIGNAL_UPSTREAM.load();
    loaded
        .is_some()
        .then(|| UpstreamConfigHandle(LEAKSIGNAL_UPSTREAM.load()))
}

lazy_static::lazy_static! {
    static ref LEAKSIGNAL_UPSTREAM: ArcSwap<Option<UpstreamConfig>> = ArcSwap::new(Arc::new(None));
}
