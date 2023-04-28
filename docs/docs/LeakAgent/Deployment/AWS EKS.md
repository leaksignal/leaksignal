---
sidebar_position: 3
---

## Deployment Options

You can get LeakAgent in AWS via:
* [Manually in EKS](..)
* Via AWS Marketplace (this document)

By default, LeakAgent will be deployed to all availability zones, however you can restrict that if needed via kubernetes `nodeSelector`s. Since there is no state or inter-communication between LeakAgent pods, you can deploy them anywhere that Prometheus can find them. It's best to keep them close to your sidecars, to avoid network latency and crossing datacenters with telemetry.

All regions are supported.

## Expected Time to Complete

5 minutes

