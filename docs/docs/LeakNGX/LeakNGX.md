# LeakNGX

LeakNGX is a native NGINX plugin to integrate LeakSignal.

## Module Files

All module files are available via `https://leakproxy.s3.us-west-2.amazonaws.com/leakngx-$LEAKSIGNAL_VERSION-$NGINX_VERSION/libleakngx.so`

Where `$LEAKSIGNAL_VERSION` is the version of LeakSignal (i.e. 0.7.2), and `$NGINX_VERSION` is a version between NGINX 1.21.6 and 1.25.2.
Note that a special version is available for NGINX 1.21.6 supporting MUSL (for alpine linux used in ingress-nginx), with a version of `$LEAKSIGNAL-VERSION-1.21.6-musl`.

## Configuring NGINX

To load LeakNGX into NGINX, you'll want to add the following directive to your `nginx.conf` main configuration:
```
load_module modules/libleakngx.so;
```
Given that `libleakngx.so` is present at `<NGINX PREFIX>/modules/libleakngx.so`

Inside of your `http` configuration block, you can add in the LeakSignal configuration:
```
# takes form of API_KEY and Ingestion endpoint. Defaults to `https://ingestion.app.leaksignal.com`
leakngx $API_KEY https://ingestion.app.leaksignal.com;
```

## Installing into NGINX Ingress Controller

LeakSignal publishes alternative container images for [NGINX Ingress Controller](https://github.com/kubernetes/ingress-nginx) at [leaksignal/ingress-nginx](https://hub.docker.com/r/leaksignal/ingress-nginx)

Tags are of the form: `$INGRESS_VERSION-$LEAKSIGNAL_VERSION`, i.e. `v1.8.1-0.7.2`. Supported versions are 1.6.4 through 1.8.1.

Example helm configuration:
```yaml
controller:
  image:
    registry: docker.io
    image: leaksignal/ingress-nginx
    tag: "v1.8.1-0.7.2"
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