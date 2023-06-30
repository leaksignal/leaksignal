---
sidebar_position: 3
---

## Deployment Options

You can get LeakAgent in AWS:
* [Manually](./)
* Via AWS Marketplace (this document)

By default, LeakAgent will be deployed to all availability zones, however you can restrict that if needed via kubernetes `nodeSelector`s. Since there is no state or inter-communication between LeakAgent pods, you can deploy them anywhere that Prometheus can find them. It's best to keep them close to your sidecars, to avoid network latency and crossing datacenters with telemetry.

All regions are supported.

## Installing via AWS Marketplace

Once you have added the project in AWS Marketplace and gotten access to the container image, you can run the image easiest via the [Helm Chart](./Helm%20Chart).

Example `values.yaml`:

```yaml
image:
  repository: 709825985650.dkr.ecr.us-east-1.amazonaws.com/leaksignal/leakagent

policies:
  example:
    categories:
      bank_routing_us:
      - internal: routing_number
    endpoints:
      - matches: "**"
        config:
          bank_routing_us: {}
```

The Helm Chart OCI Image is also hosted at `oci://709825985650.dkr.ecr.us-east-1.amazonaws.com/leaksignal/leakagent`.

It can be installed via:
```
helm upgrade --install leakagent oci://709825985650.dkr.ecr.us-east-1.amazonaws.com/leaksignal/leakagent \
  --set image.repository=709825985650.dkr.ecr.us-east-1.amazonaws.com/leaksignal/leakagent
  --version 0.3.0-helm \
  --namespace leakagent --create-namespace
```

## Expected Time to Complete

5 minutes

