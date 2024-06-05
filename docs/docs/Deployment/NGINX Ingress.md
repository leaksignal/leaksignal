---
sidebar_position: 3
---

LeakSignal publishes alternative container images for [NGINX Ingress Controller](https://github.com/kubernetes/ingress-nginx) at [leaksignal/ingress-nginx](https://hub.docker.com/r/leaksignal/ingress-nginx)

Tags are of the form: `$INGRESS_VERSION-$LEAKSIGNAL_VERSION`, i.e. `v1.8.1-0.11.2`. Supported versions are 1.6.4 through 1.8.1.

Example helm configuration:
```yaml
controller:
  image:
    registry: docker.io
    image: leaksignal/ingress-nginx
    tag: "v1.8.1-0.11.2"
    digest: null
  config:
    main-snippet:
        load_module modules/libleakngx.so;
    http-snippet:
        leakngx $API_KEY https://ingestion.app.leaksignal.com;

```

### Getting Service Names from NGINX

In your LeakSignal Policy, include the following snippet:

```yaml
local_service_name:
  ns:
    attrs:
      ngx_namespace
  sa:
    attrs:
      ngx_service_name
```

This will extract the namespace and service name of upstreams as the service name for traffic from NGINX.