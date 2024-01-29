---
sidebar_position: 2
---

## CRDs

There are two CRDs (custom resource definitions) owned by the LeakSignal Operator:

### LeaksignalIstio

LeaksignalIstio CRs (custom resources) configure LeakSignal proxies to be added to the Istio sidecars in the same namespace as the LeaksignalIstio object.

EnvoyFilter objects are created and managed by the operator to faciliate this. Pods with sidecars are automatically restarted to allow for changes to propagate.

Automatic pod refreshment can be disabled with the `spec.refreshPodsOnUpdate` value.

### ClusterLeaksignalIstio

ClusterLeaksignalIstio CRs create a default configuration for all Istio sidecars in all namespaces except for those containing a LeaksignalIstio CR.

They are cluster-scoped objects, and have the same format as LeaksignalIstio objects.

## (Cluster)LeaksignalIstio Fields

The following fields are defined by LeaksignalIstio and ClusterLeaksignalIstio objects:
```
# Required, Proxy Version String, can see all versions at https://github.com/leaksignal/leaksignal/releases
proxyVersion: 2024_01_26_19_18_07_0292938_0.9.4

# Required, SHA256 Hash of the WASM proxy module
proxyHash: fcecd3a3b099bebb432cf78e48c6f3f24a7d71b92e06b75ba5301877554960ff

# Required, API Key from Leaksignal Command dashboard or the deployment name from LeakAgent
apiKey: MY_API_KEY

# Optional. The ingestion service hostname. The default is `ingestion.app.leaksignal.com` for the public dashboard. This does not include the port.
upstreamLocation: ingestion.app.leaksignal.com

# Optional. The port for the ingestion service. The default is 443.
upstreamPort: 443

# Optional. Default is `s3/leakproxy` for public or onprem Command instances. For LeakAgent, the correct value is usually `proxy`.
proxyPrefix: s3/leakproxy

# Optional. Default is `true`. Sets whether TLS will be used for the proxy -> Command/LeakAgent communication.
tls: true

# Optional. The location of the CA certificate bundle on the Envoy Sidecar for TLS communication to the upstream Command/LeakAgent.
# For public TLS certificates, it will always be `/etc/ssl/certs/ca-certificates.crt` for Istio. (default)
# For OpenShift Service Mesh, the value is `/etc/ssl/certs/ca-bundle.crt`.
caBundle: /etc/ssl/certs/ca-certificates.crt

# Optional. Default is `true`. If set, when the Operator detects a sidecar has had it's LeakSignal configuration change, the pod is restarted.
# Envoy can load proxies without a reload okay, but doesn't do well loading in new versions of the proxy or changing the configuration on the fly.
# This option exists to avoid confusing states where one version is configured to run, but an older one is still executing.
refreshPodsOnUpdate: true

# Optional. Default is `default` which uses Google'd GRPC implementation. A value of `envoy` will use the built-in Envoy GRPC client.
grpcMode: default

# Optioal. Default is `true`. If set, L4 streaming is enabled.
enableStreaming: true

# Optional. A label selector that operates on pods. Only pods with sidecars that match all specified labels will deploy LeakSignal. The default is empty (no labels required).
workloadSelector:
  labels:
    app: my-app
```

## Examples

### Push to Leaksignal Command
This configuration will push telemetry to the LeakSignal Dashboard.

```
apiVersion: leaksignal.com/v1
kind: LeaksignalIstio
metadata:
  name: leaksignal-istio
spec:
  proxyVersion: 2024_01_26_19_18_07_0292938_0.9.4
  proxyHash: fcecd3a3b099bebb432cf78e48c6f3f24a7d71b92e06b75ba5301877554960ff
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
  proxyVersion: 2024_01_26_19_18_07_0292938_0.9.4
  proxyHash: fcecd3a3b099bebb432cf78e48c6f3f24a7d71b92e06b75ba5301877554960ff
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
  proxyVersion: 2024_01_26_19_18_07_0292938_0.9.4
  proxyHash: fcecd3a3b099bebb432cf78e48c6f3f24a7d71b92e06b75ba5301877554960ff
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
  proxyVersion: 2024_01_26_19_18_07_0292938_0.9.4
  proxyHash: fcecd3a3b099bebb432cf78e48c6f3f24a7d71b92e06b75ba5301877554960ff
  apiKey: my_policy_name

  upstreamLocation: leakagent.leakagent.svc.cluster.local
  upstreamPort: 8121
  proxyPrefix: proxy
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
  proxyVersion: 2024_01_26_19_18_07_0292938_0.9.4
  proxyHash: fcecd3a3b099bebb432cf78e48c6f3f24a7d71b92e06b75ba5301877554960ff
  apiKey: my_policy_name

  upstreamLocation: leakagent.mydomain.com
  upstreamPort: 443
  proxyPrefix: proxy
```