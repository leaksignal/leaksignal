---
sidebar_position: 1
---

## Architecture

LeakSignal Operator is a straightforward operator that translates your LeakSignal configuration settings into a working Istio integration. It manages the `EnvoyFilter` objects necessary to integrate with Istio.

The Operator is stateless and performs no network activity beyond communication with the Kubernetes Control Plane via the Kubernetes API.

## Requirements

* Kubernetes
* Istio or OpenShift Service Mesh

## Docker Images

There is a public image available:

* [`leaksignal/leaksignal-operator`](https://hub.docker.com/r/leaksignal/leaksignal-operator)

The version is not correlated with the LeakSignal version.

## Helm Chart

The easiest way to deploy the Operator is with the Helm Chart:

```
helm upgrade --install leaksignal-operator oci://registry-1.docker.io/leaksignal/leaksignal-operator \
  --version 1.3.1-helm \
  --namespace kube-system
```

The namespace can be changed if preferred.

### Upgrading Helm Chart

When upgrading the helm chart, the `helm upgrade --install` command above should still be used with the desired version.

You should also update the CRDs:
```
kubectl apply -f https://raw.githubusercontent.com/leaksignal/leaksignal-operator/master/chart/crds/leaksignal-crd.yaml https://raw.githubusercontent.com/leaksignal/leaksignal-operator/master/chart/crds/leaksignal-cluster-crd.yaml
```

### Helm Values

Check out our [GitHub Repository](https://github.com/leaksignal/leaksignal/tree/master/operator_helm) for details on Helm configurable values.

For most situations, the defaults will work right out of the box.

### Next Steps

Check out [Getting Started](./Getting%20Started) to get LeakSignal deployed!