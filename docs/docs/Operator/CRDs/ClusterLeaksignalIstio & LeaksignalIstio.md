
The following fields are defined for the `spec` of the (Cluster)LeaksignalIstio object:

```yaml

# Required. Version string for LeakSignal Proxy deployment. Can see all versions at https://github.com/leaksignal/leaksignal/releases
proxyVersion: 2025_01_20_23_02_32_4a08db1_0.12.3

# Required. Hash of the downloaded bundle for LeakSignal Proxy. Will depend on your version and deployment mechanism (nginx, envoy, WASM).
proxyHash: 13e765b6bce5ea2f1178e2e5e1ddf4ac310932928d2d138ee919207b0329ccda

# Required, API Key from the LeakSignal Command dashboard. Alternatively, the deployment name from LeakAgent.
apiKey: MY_API_KEY

# Optional. Hostname of upstream location to send metrics to. Default is `ingestion.app.leaksignal.com`.
upstreamLocation: ingestion.app.leaksignal.com

# Optional. Default is `true`. Sets whether TLS will be used for the proxy -> Command/LeakAgent communication.
tls: true

# Optional. Port of upstream ingestion. Defaults to 80/443 depending on `tls`. Recommended 8121 for LeakAgent.
upstreamPort: 443

# Optional. The location of the CA certificate bundle on the Envoy Sidecar for TLS communication to the upstream Command/LeakAgent.
# For public TLS certificates, it will always be `/etc/ssl/certs/ca-certificates.crt` for Istio. (default)
# For OpenShift Service Mesh, the value is `/etc/ssl/certs/ca-bundle.crt`.
# Not used in Native deployments.
caBundle: /etc/ssl/certs/ca-certificates.crt

# Optional. For WASM mode, redeploys all pods with Istio sidecars affected by a LeakSignal Proxy upgrade. This provides more consistent behavior. Default is `true`.
refreshPodsOnUpdate: true

# Optional. Detects pods that should have leaksignal deployed, but dont, and restarts them.
refreshPodsOnStale: true

# Optional. Default is `default` which uses Google's GRPC implementation. A value of `envoy` will use the built-in Envoy GRPC client. Only affects WASM.
grpcMode: default

# Optional. If `true` (default), then L4 streams are also scanned by LeakSignal Proxy.
enableStreaming: true

# Optional. If `true` (default), istio-proxy containers are updated to a corresponding image with support for dynamic plugins, and the native LeakSignal Proxy module is installed.
native: true

# Optional. If `true` (default), if LeakSignal Proxy has a failure, then all traffic is routed around it.
failOpen: true

# Optional. If set, use an alternate name for created EnvoyFilter objects, to allow multiple LeaksignalIstio objects in one namespace.
istioName: "leaksignal-istio"

# Optional. If no tag is specified, it is inferred from the existing proxy image on each given pod.
nativeRepo: "leaksignal/istio-proxy"

# Optional. Specifies an HTTP prefix to pull LeakSignal Proxy from.
proxyPullLocation: "https://leakproxy.s3.us-west-2.amazonaws.com/"

# Optional. A label selector that operates on pods. Only pods with sidecars that match all specified labels will deploy LeakSignal. The default is empty (no labels required).
workloadSelector:
  labels:
    app: my-app
```