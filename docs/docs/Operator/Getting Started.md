---
sidebar_position: 2
---

## CRDs

There are two CRDs (custom resource definitions) owned by the LeakSignal Operator:

### LeaksignalIstio

[LeaksignalIstio](./CRDs/ClusterLeaksignalIstio%20&%20LeaksignalIstio) CRs (custom resources) configure LeakSignal proxies to be added to the Istio sidecars in the same namespace as the LeaksignalIstio object.

EnvoyFilter objects are created and managed by the operator to faciliate this. Pods with sidecars are automatically restarted to allow for changes to propagate (WASM only).

Automatic pod refreshment can be disabled with the `spec.refreshPodsOnUpdate` value.

### ClusterLeaksignalIstio

[ClusterLeaksignalIstio](./CRDs/ClusterLeaksignalIstio%20&%20LeaksignalIstio) CRs create a default configuration for all Istio sidecars in all namespaces except for those containing a LeaksignalIstio CR.

They are cluster-scoped objects, and have the same format as LeaksignalIstio objects.

## Examples

### Push to Leaksignal Command
This configuration will push telemetry to the LeakSignal Dashboard.

```
apiVersion: leaksignal.com/v1
kind: LeaksignalIstio
metadata:
  name: leaksignal-istio
spec:
  proxyVersion: 2025_01_20_23_02_32_4a08db1_0.12.3
  proxyHash: 13e765b6bce5ea2f1178e2e5e1ddf4ac310932928d2d138ee919207b0329ccda
  apiKey: MY_API_KEY
```

### Push to Entrprise Leaksignal Command On-Prem
This configuration will push telemetry to an on-prem deployment of LeakSignal Dashboard.

```
apiVersion: leaksignal.com/v1
kind: LeaksignalIstio
metadata:
  name: leaksignal-istio
spec:
  proxyVersion: 2025_01_20_23_02_32_4a08db1_0.12.3
  proxyHash: 13e765b6bce5ea2f1178e2e5e1ddf4ac310932928d2d138ee919207b0329ccda
  apiKey: MY_API_KEY
  upstreamLocation: ingestion.leaksignal.mydomain.com
```

Or with OpenShift Service Mesh:

```
apiVersion: leaksignal.com/v1
kind: LeaksignalIstio
metadata:
  name: leaksignal-istio
spec:
  proxyVersion: 2025_01_20_23_02_32_4a08db1_0.12.3
  proxyHash: 13e765b6bce5ea2f1178e2e5e1ddf4ac310932928d2d138ee919207b0329ccda
  apiKey: MY_API_KEY
  upstreamLocation: ingestion.leaksignal.mydomain.com
  caBundle: /etc/ssl/certs/ca-bundle.crt
```

### Push to local LeakAgent
This configuration will push telemetry to a same-cluster LeakAgent deployment.

```
apiVersion: leaksignal.com/v1
kind: LeaksignalIstio
metadata:
  name: leaksignal-istio
spec:
  proxyVersion: 2025_01_20_23_02_32_4a08db1_0.12.3
  proxyHash: 13e765b6bce5ea2f1178e2e5e1ddf4ac310932928d2d138ee919207b0329ccda
  apiKey: my_policy_name

  upstreamLocation: leakagent.leakagent.svc.cluster.local
  upstreamPort: 8121
  tls: false
```

### Push to remote LeakAgent
This configuration will push telemetry to a remote LeakAgent deployment behind an Ingress providing TLS termination.

```
apiVersion: leaksignal.com/v1
kind: LeaksignalIstio
metadata:
  name: leaksignal-istio
spec:
  proxyVersion: 2025_01_20_23_02_32_4a08db1_0.12.3
  proxyHash: 13e765b6bce5ea2f1178e2e5e1ddf4ac310932928d2d138ee919207b0329ccda
  apiKey: my_policy_name

  upstreamLocation: leakagent.mydomain.com
  upstreamPort: 443
```