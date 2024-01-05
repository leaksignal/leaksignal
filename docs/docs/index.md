---
sidebar_position: 1
title: Getting Started
---

<p align="center">🔍 There are all kinds of sensitive data flowing through my services, but I don’t know which ones or what data. 🤷</p>

LeakSignal provides observability metrics (or [statistics](https://bit.ly/3Twj9ca)) for sensitive data contained in request/response content. LeakSignal metrics can be consumed by Prometheus, pushed as OpenTelemetry, or collected in a centralized dashboard - giving operations engineers (SRE, DevOps, Platform Eng. etc) a new security tool to help combat API exploits, unknown misconfigurations and sensitive data leakage.

## Features

* Fast, inline Layer 7 request/response analysis.
* Easy to configure rules ("L7 policy") for detecting and analyzing sensitive data (e.g. PII) leakage.
  * Detect part numbers, account numbers, patient info, grades, dates, email addresses, large arrays, etc. You can write your own or use our constantly evolving <a href="https://github.com/leaksignal/leaksignal/tree/master/examples/policies">ruleset</a> library (contributions welcome).
* Cloud dashboard with policy editor, monitoring, and alerting.
* Analysis metrics can be exposed via Envoy and thus reflected wherever Envoy metrics are configured to land (OpenTelemetry, Prometheus, etc.)

## Installation

LeakSignal installs in moments as a WASM filter for Envoy, Istio, or any proxy/API gateway that supports Proxy-WASM. No CRD, no additional containers or sidecars, no other dependencies, just a WASM binary. See Getting Started below.

## Table of Contents

* [Overview](#overview)
  * [Sentry](#leaksignal-sentry)
  * [Command](#leaksignal-command)
  * [Implementation](#implementation)
* [Getting Started](#getting-started)
  * [Getting Started with a demo application](#getting-started-with-a-demo-application)
  * [Getting started with an existing setup](#getting-started-with-existing-setup)
    * [Configuration Quickstarts](#quickstarts)
  * [Verify Setup](#verify-proper-setup)
  * [View Metrics with Prometheus & Grafana](#view-metrics-prometheus--grafana)
  * [View Metrics in Command cloud dashboard](#view-metrics-command)
  * [Test and configure L7 Policy](#test-and-configure-l7-policy)
* [Community / How to Contribute](#community--how-to-contribute)
* [License](#license)
* [Commercial Support](#commercial-support)

## Overview

LeakSignal detects sensitive data within mesh traffic. Analysis and processing of traffic is handled inline, allowing engineers to understand sensitive data emission without sending, storing or viewing the sensitive data.
<p align="center">
  <img style={{"max-width": "75%"}} src="https://github.com/leaksignal/leaksignal/raw/master/assets/mesh-overview2.png" />
</p>

### LeakSignal SENTRY

LeakSignal establishes a framework and delivery mechanism for composable traffic analysis functions within a WASM VM. Sentry is the bytecode that allows for sensitive data analysis across request and response traffic in real-time.
<p align="center">
  <img style={{"max-width": "75%"}} src="https://github.com/leaksignal/leaksignal/raw/master/assets/filter-overview2.png" />
</p>
The following functionality can be enabled through the Layer7 Policy:

* Sensitive Data Observability
* Data Access by IP or Token
* Exfiltration Mitigation
* Data Access Auditing
* Prometheus and OTEL metrics
* Dashboard visualization (histogram, heatmap) and alerting via SMS or email

### LeakSignal COMMAND

LeakSignal Command (the cloud dashboard) provides visibility of data types and sends you SMS or email alerts when abnormal or unauthorized data access occurs.
<p align="center">
  <img style={{"max-width": "75%"}} src="https://github.com/leaksignal/leaksignal/raw/master/assets/command-overview1.png" />
</p>

### Implementation

Built with Rust and deployed as WebAssembly, LeakSignal natively runs on proxies and API Gateways supporting [Proxy-WASM](https://bit.ly/3s8SeYg). The current implementation is tested with Envoy, which is the underlying data management plane in most service mesh offerings.

LeakSignal analysis can be setup in the following modes:

* All metrics and configuration stay local in your environment
* All metrics and configuration go to LeakSignal COMMAND.
  * Sensitive data are sent to COMMAND by default.
  * Specific endpoints, match rules, or the entire policy can opt-in to send raw sensitive data, low-bit subsets of SHA-256 hashes for low-entropy data (i.e. credit cards, phone numbers), or no representation of the matched data at all.

## Getting Started

### Getting Started with a Demo application

If you're looking to kick the tires with a demo setup, you have 2 options:

1. [Simple Envoy Ingress controller for K8s cluster](https://bit.ly/3s8zAzE).
    * LeakSignal is preinstalled with policy and test applications/services.
2. [Google's Online Boutique microservices demo for Istio](https://bit.ly/3TATdvI).
    * Follow along with the Istio install and then add LeakSignal.

### Getting Started with Existing Setup

If you already have an environment up and running (Standalone Envoy, K8s, or Istio) where you'd like to install LeakSignal, use the following quick starts.

#### Quickstarts

<details>
  <summary>Raw Configs</summary>

1. [Register for an account](https://bit.ly/3MFtlgd) (Note: you don't need an account if you plan on only sending metrics to prometheus)
2. Diff your Envoy or Istio configs against the [examples](https://github.com/leaksignal/leaksignal/tree/master/examples/).
3. Add your API key and Deployment name to your new config.

</details>

<details>
  <summary>Envoy Docker Quickstart</summary>

Docker commands to run an Envoy proxy with LeakSignal installed.

1. [Register for an account](https://bit.ly/3MFtlgd)
2. Get your API key by clicking "Deployments" in the left hand navigation.
3. Create a simple barebones deployment by clicking "Create Deployment" on the Deployments page.
4. Replace YOUR-API-KEY below with the values in LeakSignal Command.

```bash
FROM envoyproxy/envoy-dev:0b1c5aca39b8c2320501ce4b94fe34f2ad5808aa
RUN curl -O https://raw.githubusercontent.com/leaksignal/leaksignal/master/examples/envoy/envoy_command_remote_wasm.yaml > envoy_raw.yaml
RUN API_KEY="YOUR-API-KEY" envsubst < envoy_raw.yaml > /etc/envoy.yaml
RUN chmod go+r /etc/envoy.yaml
CMD ["/usr/local/bin/envoy", "-c", "/etc/envoy.yaml"]
```
>
> * Go to Deployments -> Deployment Name and learn more about the L7 Policy that is currently running.
> * [View metrics in COMMAND](#view-metrics-command)

</details>

<details>
  <summary>Envoy-Local Docker Quickstart (no cloud connection)</summary>

Docker commands to run an Envoy proxy with LeakSignal installed.

* This configuration runs LeakSignal in "local" mode where metrics are only exported in the running Envoy instance.
* The LeakSignal L7 Policy is contained in the Envoy yaml configuration.
* LeakSignal API Key and deployment name are not needed.

```bash
FROM envoyproxy/envoy-dev:0b1c5aca39b8c2320501ce4b94fe34f2ad5808aa
RUN curl -O https://raw.githubusercontent.com/leaksignal/leaksignal/master/examples/envoy/envoy_local.yaml > /etc/envoy.yaml
RUN curl -O https://ingestion.app.leaksignal.com/s3/leakproxy/2024_01_05_01_38_43_4beed93_0.9.0/leaksignal.wasm
RUN chmod go+r /etc/envoy.yaml
CMD ["/usr/local/bin/envoy", "-c", "/etc/envoy.yaml"]
```

> * [Verify everything is setup correctly](#verify-proper-setup).
> * Test and configure L7 Policy for your environment
> * [View prometheus metrics in grafana](#view-metrics-prometheus--grafana)

Use the [test environment](https://bit.ly/3s8zAzE) to see a working example. Your sensitive data labels and counts will be exported as Envoy metrics.
</details>

<details>
  <summary>Istio</summary>

Install LeakSignal across all Istio sidecar proxies with the following:

1. [Register for an account](https://bit.ly/3MFtlgd)
2. Grab your API key from the "Configure" tab for your new deployment
3. Replace YOUR-API-KEY below with the API key in LeakSignal Command.

```bash
# Apply the following leaksignal.yaml to deploy the filter
export API_KEY="YOUR-API-KEY" && \
curl https://raw.githubusercontent.com/leaksignal/leaksignal/master/examples/istio/leaksignal.yaml | \
envsubst | \
kubectl apply -f -
```

> Go to Deployments -> Deployment Name and learn more about the L7 Policy that is currently running.

</details>

<details>
  <summary>Istio (single namespace)</summary>

Install LeakSignal across all Istio sidecar proxies in a given namespace proxies with the following:

1. [Register for an account](https://bit.ly/3MFtlgd)
2. Grab your API key from the "Configure" tab for your new deployment
3. Replace YOUR-API-KEY below with the API key in LeakSignal Command.
4. Replace YOUR-NAMESPACE below with the namespace you would like to deploy LeakSignal to.

```bash
# Apply the following leaksignal.yaml to deploy the filter
export API_KEY="YOUR-API-KEY" && \
curl https://raw.githubusercontent.com/leaksignal/leaksignal/master/examples/istio/leaksignal_ns.yaml | \
envsubst | \
kubectl apply -n YOUR-NAMESPACE -f -
```

> Go to Deployments -> Deployment Name and learn more about the L7 Policy that is currently running.

</details>

<details>
  <summary>Istio-Local (no cloud metrics)</summary>

Install LeakSignal across all Istio sidecar proxies with the following.

* This configuration runs LeakSignal in "local" mode where metrics are only exported in the running Envoy instance.
* The LeakSignal L7 Policy is contained in the Envoy yaml configuration.
* LeakSignal API Key and deployment name are not needed.

A connection to the cloud is still necessary to pull the WASM proxy, but no metrics or sensitive data are uploaded.

```bash
# Apply the following leaksignal.yaml to deploy the filter
curl https://raw.githubusercontent.com/leaksignal/leaksignal/master/examples/istio/leaksignal_local.yaml | kubectl apply -f -
```
>
> * [Verify everything is setup correctly](#verify-proper-setup).
> * Test and configure L7 Policy for your environment
> * [View prometheus metrics in grafana](#view-metrics-prometheus--grafana)

</details>

#### Verify Proper Setup

After you've installed the LeakSignal filter, you can check the logs to see how things are running:

For Envoy standalone run:

```bash
tail -f /var/log/envoy.log
```

For Kubernetes run:

```bash
kubectl get pods
#find the envoy pod and use it below
kubectl logs -f [envoy podname]
```

For Istio run:

```bash
kubectl -n istio-system get pods
kubectl -n istio-system logs istio-ingressgateway-abc123
#if you see no policy loaded, make sure your api key and deployment name is correct.
kubectl -n istio-system describe EnvoyFilter | grep api_key
```

In all cases you should see messsages with "leaksignal" in the logs. Use those to understand if things are setup correctly. Note that you may see messages like `createWasm: failed to load (in progress) from https://ingestion.app...` if loading the wasm file remotely. This is a known issue and the wasm filter is functioning properly.

### View Metrics (Prometheus & Grafana)

Prometheus is capable of ingesting LeakSignal metrics. You can configure your policy to alert on specific data types to detect spikes in emission of data or edge cases like the signature of a known RCE. (If you don't have or want to use Prometheus skip to the next step).

Here's an example from our [k8s test environment](https://bit.ly/3s8zAzE) where grafana displays LeakSignal metrics from prometheus:

LeakSignal defines 2 new metrics in Grafana:

1. Sensitive Data per Minute (SDPM)
2. Exploits per Minute (EPM)

<img src="https://github.com/leaksignal/leaksignal/raw/master/assets/grafana-overview.png" />

These metrics are visible for any API endpoint configured in the LeakSignal policy.

### View Metrics (COMMAND)

Once you login to LeakSignal COMMAND, you'll see the Sensitive Data Overview as the default screen.

The following example data is from the k8s [test environment](https://bit.ly/3s8zAzE).
<img src="https://github.com/leaksignal/leaksignal/raw/master/assets/sd_detail.png" width="750" />

This chart shows the emission of sensitive data and exploited logic as defined by [the L7 policy](https://github.com/leaksignal/testing-environments/blob/main/kubernetes/envoy.yaml#L180).

The following test pages are used to generate the alerts

* [ssn001.html](https://github.com/leaksignal/testing-environments/blob/main/servers/node/public/node/ssn001.html) contains PII data such as Social Security and Phone Numbers. (green and purple)
* [root.html](https://github.com/leaksignal/testing-environments/blob/main/servers/node/public/node/root.html) is an example of leaked configuration file or any response with the word "root" in it.
* [ls.html](https://github.com/leaksignal/testing-environments/blob/main/servers/node/public/node/ls.html) and [ifconfig.html](https://github.com/leaksignal/testing-environments/blob/main/servers/node/public/node/ifconfig.html) are examples of a response that contain results from a system command being executed on the server (RCE).

Scroll down to the data grid and click on a Response ID to examine the alerts that were generated.
<img src="https://github.com/leaksignal/leaksignal/raw/master/assets/alerts_w_page.png" width="750" />

Click Heat Map in the left hand nav for a complete view of how sensitive data is accessed by IP addresses and/or authentication tokens
<img src="https://github.com/leaksignal/leaksignal/raw/master/assets/heatmap.png" width="750" />

More docs coming soon!

### Test and configure L7 Policy

After you've verified that the filter is running, you can configure the policy to check for specific sensitive data types or patterns. For examples of preconfigured and performance tested policies, see [LeakSignal Policies](Policy)

## Community / How to Contribute

* Code contribution guidelines (Coming soon)

## Commercial support

- Leaksignal, Inc offers support and self-hosted versions of the cloud dashboard. Contact <sales@leaksignal.com>.

## License

Copyright 2022 LeakSignal, Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
