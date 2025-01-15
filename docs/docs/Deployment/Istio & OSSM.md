---
sidebar_position: 1
---

The easiest way to deploy for [Istio](https://istio.io/) and [OpenShift Service Mesh](https://www.redhat.com/en/technologies/cloud-computing/openshift/what-is-openshift-service-mesh) is via the [LeakSignal Operator](/Operator/Deployment).

## Deploying the Operator

LeakSignal Operator is available on [OperatorHub](https://operatorhub.io/operator/leaksignal-operator), or as a [Certified Operator on OpenShift OperatorHub](https://catalog.redhat.com/software/container-stacks/detail/64f9f47e9c7ac3eb6ed9605d).

You can also install the operator via a Helm Chart:
```bash
$ helm upgrade --install leaksignal-operator oci://registry-1.docker.io/leaksignal/leaksignal-operator \
  --version 0.12.2-helm \
  --namespace leaksignal-operator \
  --create-namespace
```

If updating from a previous version via Helm, make sure to manually update the CRDs:
```bash
$ kubectl apply -f https://raw.githubusercontent.com/leaksignal/leaksignal-operator/v1.8.1/crds/leaksignal-crd.yaml https://raw.githubusercontent.com/leaksignal/leaksignal-operator/v1.8.1/crds/leaksignal-cluster-crd.yaml
```

You can find the code, CRDs, and Helm Charts on [GitHub](https://github.com/leaksignal/leaksignal-operator) and more documentation [Here](/Operator/Deployment).

## Native vs WASM

For Istio & OSSM, there are two available deployment modes: Native & WASM.

* **WASM** provides a full sandbox for LeakSignal to run in, providing redundancy in case of failure.
* **Native** provides more performance for less resource usage, and in some environments, more reliable networking. It also supports live hot-reloading for new versions of the proxy unlike WASM. It is not fully supported on OSSM though.

#### So which should I pick?

On OSSM, **WASM** is recommended. Native mode requires a custom Envoy proxy image to support dynamically loaded Native proxy modules which is not currently available for OSSM, and some OSSM features will not work with the LeakSignal-provided proxy image.

If you are concerned with deploying executable code into all of your Service Mesh Sidecars, **WASM** might also be the right choice for you, as it provides a fully sandboxed environment.

If you want to minimize resource usage, get latency improvements, and more reliable networking to Command, **Native** is a great choice.

## Deploying the Proxy

With the Operator deployed, you can now deploy LeakSignal Proxy.

### Single Namespace

To deploy in a single namespace, you can create a **LeakSignalIstio**:
```yaml
apiVersion: leaksignal.com/v1
kind: LeaksignalIstio
metadata:
  name: leaksignal-istio
spec:
  # Version information is available at https://github.com/leaksignal/leaksignal/releases
  proxyVersion: 2025_01_15_21_20_54_a5482a4_0.12.2
  proxyHash: 2fd11b9808b9b5fd0d102f81a6686821d5d8c305dd746418c5e48b01a7163d68
  # from Command, or the Deployment name in LeakAgent
  apiKey: MY_API_KEY
```

### Single Namespace (WASM)

If you want to deploy with **Native** mode, there will be a different `proxyHash` and a `native: false` flag:
```yaml
apiVersion: leaksignal.com/v1
kind: LeaksignalIstio
metadata:
  name: leaksignal-istio
spec:
  # Version information is available at https://github.com/leaksignal/leaksignal/releases
  proxyVersion: 2025_01_15_21_20_54_a5482a4_0.12.2
  proxyHash: c7b0f20024e56df1dcb3c32f2c64799a049e8e661cf1ba318fcc932f43d9eb9b
  native: false
  # from Command, or the Deployment name in LeakAgent
  apiKey: MY_API_KEY
```

### Cluster-Wide

If you would like to deploy LeakSignal Proxy in all namespaces, you can create a **ClusterLeaksignalIstio**. Any namespaces that contain a **LeaksignalIstio** will not be affected.

```yaml
apiVersion: leaksignal.com/v1
kind: ClusterLeaksignalIstio
metadata:
  name: leaksignal-istio
spec:
  # Version information is available at https://github.com/leaksignal/leaksignal/releases
  proxyVersion: 2025_01_15_21_20_54_a5482a4_0.12.2
  proxyHash: 2fd11b9808b9b5fd0d102f81a6686821d5d8c305dd746418c5e48b01a7163d68
  # from Command, or the Deployment name in LeakAgent
  apiKey: MY_API_KEY
```
