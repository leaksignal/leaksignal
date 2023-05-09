---
sidebar_position: 1
---

## Architecture

LeakAgent deploys as a stateless, horizontally scalable Kubernetes deployment. Envoy reports all LeakSignal telemetry to LeakAgent instead of the cloud (LeakSignal Command). LeakAgent takes that telemetry, and exposes a subset of it to Prometheus.

LeakAgent makes no outgoing connections, and only serves to distribute policies to LeakSignal proxies, receive telemetry, and emit serve that telemetry to Prometheus. LeakAgent requires no special permissions, roles, or access.

<img src="https://github.com/leaksignal/leaksignal/raw/master/assets/leakagent-architecture.png" />

## Requirements
* Prometheus setup to fetch metrics from LeakAgent
* Kubernetes
* Istio is recommended but not required

## Examples

* [Deployment Example with Prometheus Operator integration](https://github.com/leaksignal/leaksignal/tree/master/examples/leakagent)
* [Sample Grafana Dashboards](https://github.com/leaksignal/leaksignal/tree/master/examples/leakagent/grafana)
* [Istio Integration](https://github.com/leaksignal/leaksignal/blob/master/examples/istio/leaksignal_agent.yaml)
* [Envoy Direct Integration](https://github.com/leaksignal/leaksignal/blob/master/examples/envoy/envoy_agent.yaml)

## Docker Images

LeakAgent is available as two standard docker images:
* [`leaksignal/leakagent`](https://hub.docker.com/r/leaksignal/leakagent)
* [`leaksignal/leakagent_bundle`](https://hub.docker.com/r/leaksignal/leakagent_bundle)

The `_bundle` variant includes the leaksignal proxy under `ORIGIN/proxy/leaksignal.wasm` for reference from Envoy.

## Configuration

LeakAgent has the following environment variables:
* `LS_PROMETHEUS_BIND`: A socket address to bind for exporting prometheus metrics. Default: `0.0.0.0:9176`
* `LS_INGESTION_BIND`: A socket address to bind for LeakSignal to connect to. Default: `0.0.0.0:8121`
* `LS_BUNDLE_DIR`: A directory (with or without trailing slash) in which to serve bundled WASM files from. No default.
* `LS_CONFIG_PATH`: A file path that points directly to the configuration file for LeakAgent.

If `LS_BUNDLE_DIR` is set and non-empty **and** `proxy_source` in the configuration file is empty, then `proxy_source` is defaulted to bundling from that directory.

Configuration is hot-reloadable without restarting the LeakAgent pods.

## Deployment Patterns and Scaling

LeakAgent is a lightweight and stateless and a passive receiver of telemetry, which is ultimately scraped by Prometheus. As a result, you can deploy as much as you need wherever you need, with no concern of scaling constraints. The bottleneck you will using LeakAgent is network bandwidth, and so it is advisable to have a few pods available for every region your cluster is running in.

### Configuration Format
```yaml
policies:
  # the deployment name here is used as both a deployment name AND the api_key for proxies to connect.
  <DEPLOYMENT NAME>:
    # YAML LeakSignal policy
    <POLICY>
proxy_source:
  type: <bundled|remote_passthrough>
  origin: <HTTP(S) origin & path> # only for `remote_passthrough` mode
  path: <File path to directory for bundles> # only for `bundled` mode
```

### Example Configuration
Environment:
```
LS_BUNDLE_DIR=/bundle
LS_CONFIG_PATH=/config/config.yaml
```

Configuration:
```yaml
policies:
  example:
    categories:
      bank_routing_us:
        Matchers:
          - regex: "(?-u:\\b)(0[0-9]|11|12|2[1-9]|30|31|32|6[1-9]|70|71|72|80)\\d{7}(?-u:\\b)"
          - and:
            - internal: routing_number
    endpoints:
      - matches: "**"
        config:
          bank_routing_us: {}
```

## Pointing Istio/Envoy to LeakAgent

A given configuration pointing to the cloud or elsewhere will have an upstream Envoy cluster named `leaksignal_infra` (by default) similar to:
```yaml
name: leaksignal_infra
type: STRICT_DNS
http2_protocol_options: {}
dns_lookup_family: V4_PREFERRED
lb_policy: ROUND_ROBIN
load_assignment:
  cluster_name: leaksignal_infra0
  endpoints:
  - lb_endpoints:
    - endpoint:
        address:
          socket_address:
            address: ingestion.app.leaksignal.com
            port_value: 443
transport_socket:
  name: envoy.transport_sockets.tls
  typed_config:
    "@type": type.googleapis.com/envoy.extensions.transport_sockets.tls.v3.UpstreamTlsContext
    sni: ingestion.app.leaksignal.com
    common_tls_context:
      validation_context:
        match_typed_subject_alt_names:
        - san_type: DNS
          matcher:
            exact: ingestion.app.leaksignal.com
        trusted_ca:
          filename: /etc/ssl/certs/ca-certificates.crt
```

To point such a cluster to a local LeakAgent deployment in the `leakagent` namespace, replace with the following:
```yaml
name: leaksignal_infra
type: STRICT_DNS
http2_protocol_options: {}
dns_lookup_family: V4_PREFERRED
lb_policy: ROUND_ROBIN
load_assignment:
  cluster_name: leaksignal_infra0
  endpoints:
  - lb_endpoints:
    - endpoint:
        address:
          socket_address:
            address: ingestion.leakagent.svc.cluster.local
            port_value: 8121

```

See [examples](#examples) for more details.

## Getting Data Into Prometheus

LeakAgent exposes Prometheus metrics on port 9176 (by default) for collection.

The deployment example in [examples](#examples) includes a [Prometheus Operator](https://github.com/prometheus-operator/prometheus-operator) integration.

You'll need to make sure Prometheus Operator looks for `ServiceMonitor` CRs in the `leakagent` namespace as shown in the example. Alternatively, move the `ServiceMonitor` to the namespace in which Prometheus Operator is installed/already looking. I.e. `default` for a direct deployment of Prometheus Operator or `monitoring` for [kube-prometheus](https://github.com/prometheus-operator/kube-prometheus) (recommended).

Once data is being sent to Prometheus, you should be able to query it in an attached Grafana deployment or similar.