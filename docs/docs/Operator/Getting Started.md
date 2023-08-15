---
sidebar_position: 2
---

## CRDs

There are two CRDs (custom resource definitions) owned by the LeakSignal Operator:

### LeaksignalIstio

LeaksignalIstio CRs (custom resources) configure LeakSignal proxies to be added to the Istio sidecars in the same namespace as the LeaksignalIstio object.

EnvoyFilter objects are created and managed by the operator to faciliate this. Pods with sidecars are automatically restarted to allow for changes to propagate.

Automatic pod refreshment can be disabled with the `spec.refresh_pods_on_update` value.

### ClusterLeaksignalIstio

ClusterLeaksignalIstio CRs create a default configuration for all Istio sidecars in all namespaces except for those containing a LeaksignalIstio CR.

They are cluster-scoped objects, and have the same format as LeaksignalIstio objects.

## (Cluster)LeaksignalIstio Fields

The following fields are defined by LeaksignalIstio and ClusterLeaksignalIstio objects:
```
# Required, Proxy Version String, can see all versions at https://github.com/leaksignal/leaksignal/releases
proxy_version: 2023_08_15_15_06_31_cec8584_0.4.0

# Required, SHA256 Hash of the WASM proxy module
proxy_hash: 01550775d2f45f8358dc53d660a274555eae6bf3525dfad95b306537791c8f82

# Required, API Key from Leaksignal Command dashboard or the deployment name from LeakAgent
api_key: MY_API_KEY

# Optional. The ingestion service hostname. The default is `ingestion.app.leaksignal.com` for the public dashboard. This does not include the port.
upstream_location: ingestion.app.leaksignal.com

# Optional. The port for the ingestion service. The default is 443.
upstream_port: 443

# Optional. Default is `s3/leakproxy` for public or onprem Command instances. For LeakAgent, the correct value is usually `proxy`.
proxy_prefix: s3/leakproxy

# Optional. Default is `true`. Sets whether TLS will be used for the proxy -> Command/LeakAgent communication.
tls: true

# Optional. The location of the CA certificate bundle on the Envoy Sidecar for TLS communication to the upstream Command/LeakAgent.
# For public TLS certificates, it will always be `/etc/ssl/certs/ca-certificates.crt` for Istio. (default)
# For OpenShift Service Mesh, the value is `/etc/ssl/certs/ca-bundle.crt`.
ca_bundle: /etc/ssl/certs/ca-certificates.crt

# Optional. Default is `true`. If set, when the Operator detects a sidecar has had it's LeakSignal configuration change, the pod is restarted.
# Envoy can load proxies without a reload okay, but doesn't do well loading in new versions of the proxy or changing the configuration on the fly.
# This option exists to avoid confusing states where one version is configured to run, but an older one is still executing.
refresh_pods_on_update: true
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
  proxy_version: 2023_08_15_15_06_31_cec8584_0.4.0
  proxy_hash: 01550775d2f45f8358dc53d660a274555eae6bf3525dfad95b306537791c8f82
  api_key: MY_API_KEY
```

### Push to Entrprise Leaksignal Command On-Prem
This configuration will push telemetry to an on-prem deployment of LeakSignal Dashboard.

```
apiVersion: leaksignal.com/v1
kind: LeaksignalIstio
metadata:
  name: leaksignal-istio
spec:
  proxy_version: 2023_08_15_15_06_31_cec8584_0.4.0
  proxy_hash: 01550775d2f45f8358dc53d660a274555eae6bf3525dfad95b306537791c8f82
  api_key: MY_API_KEY
  upstream_location: ingestion.leaksignal.mydomain.com
```

Or with OpenShift Service Mesh:

```
apiVersion: leaksignal.com/v1
kind: LeaksignalIstio
metadata:
  name: leaksignal-istio
spec:
  proxy_version: 2023_08_15_15_06_31_cec8584_0.4.0
  proxy_hash: 01550775d2f45f8358dc53d660a274555eae6bf3525dfad95b306537791c8f82
  api_key: MY_API_KEY
  upstream_location: ingestion.leaksignal.mydomain.com
  ca_bundle: /etc/ssl/certs/ca-bundle.crt
```

### Push to local LeakAgent
This configuration will push telemetry to a same-cluster LeakAgent deployment.

```
apiVersion: leaksignal.com/v1
kind: LeaksignalIstio
metadata:
  name: leaksignal-istio
spec:
  proxy_version: 2023_08_15_15_06_31_cec8584_0.4.0
  proxy_hash: 01550775d2f45f8358dc53d660a274555eae6bf3525dfad95b306537791c8f82
  api_key: my_policy_name

  upstream_location: leakagent.leakagent.svc.cluster.local
  upstream_port: 8121
  proxy_prefix: proxy
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
  proxy_version: 2023_08_15_15_06_31_cec8584_0.4.0
  proxy_hash: 01550775d2f45f8358dc53d660a274555eae6bf3525dfad95b306537791c8f82
  api_key: my_policy_name

  upstream_location: leakagent.mydomain.com
  upstream_port: 443
  proxy_prefix: proxy
```