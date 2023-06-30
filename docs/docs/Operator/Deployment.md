---
sidebar_position: 1
---

## Architecture

LeakSignal Operator is a straightforward operator that translates your LeakSignal configuration settings into a working Istio integration. It manages the `EnvoyFilter` objects necessary to integrate with Istio.

The Operator is stateless and performs no network activity beyond communication with the Kubernetes Control Plane via the Kubernetes API.

## Requirements

* Kubernetes
* Istio or OpenShift Service Mesh

## Examples

* [Deployment Example with Prometheus Operator integration](https://github.com/leaksignal/leaksignal/tree/master/examples/leakagent)
* [Sample Grafana Dashboards](https://github.com/leaksignal/leaksignal/tree/master/examples/leakagent/grafana)
* [Istio Integration](https://github.com/leaksignal/leaksignal/blob/master/examples/istio/leaksignal_agent.yaml)
* [Envoy Direct Integration](https://github.com/leaksignal/leaksignal/blob/master/examples/envoy/envoy_agent.yaml)

## Docker Images

There is a public image available:

* [`leaksignal/leaksignal-operator`](https://hub.docker.com/r/leaksignal/leaksignal-operator)

The version is not correlated with the LeakSignal version.

## Helm Chart

The easiest way to deploy the Operator is with the Helm Chart:

```
helm upgrade --install leaksignal-operator oci://registry-1.docker.io/leaksignal/leaksignal-operator \
  --version 1.0.0-helm \
  --namespace kube-system
```

The namespace can be changed if preferred.

### Helm Values

Check out our [GitHub Repository](https://github.com/leaksignal/leaksignal/tree/master/operator_helm) for details on Helm configurable values.

For most situations, the defaults will work right out of the box.