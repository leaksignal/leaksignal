---
sidebar_position: 1
title: Getting Started
---

<p align="center">🔍 There are all kinds of sensitive data flowing through my services, but I don’t know which ones or what data. 🤷</p>

LeakSignal provides observability metrics (or [statistics](https://bit.ly/3Twj9ca)) for sensitive data contained in request/response content. LeakSignal metrics can be consumed by Prometheus or collected in Command, our centralized dashboard - giving operations engineers (SRE, DevOps, Platform Eng. etc) a new security tool to help combat API exploits, unknown misconfigurations and sensitive data leakage.

## Features

* Fast, inline Layer 7 request/response analysis.
* Easy to configure rules ("L7 policy") for detecting and analyzing sensitive data (e.g. PII) leakage.
  * Detect part numbers, account numbers, patient info, grades, dates, email addresses, large arrays, etc. You can write your own or use our constantly evolving <a href="https://github.com/leaksignal/leaksignal/tree/master/examples/policies">ruleset</a> library (contributions welcome).
* Cloud dashboard with policy editor, monitoring, and alerting.
* Analysis metrics can be exposed via Envoy and thus reflected wherever Envoy metrics are configured to land (OpenTelemetry, Prometheus, etc.)

## Installation

LeakSignal can deploy in a variety of environments -- Istio, OpenShift Service Mesh, NGINX/NGINX Ingress, Lambda, and more.

See detailed installation instructions [here](./Deployment/Istio%20&%20OSSM)

## Overview

LeakSignal detects sensitive data within mesh traffic. Analysis and processing of traffic is handled inline, allowing engineers to understand sensitive data emission without sending, storing or viewing the sensitive data.
<p align="center">
  <img style={{"max-width": "75%"}} src="https://github.com/leaksignal/leaksignal/raw/master/assets/mesh-overview2.png" />
</p>

### LeakSignal Proxy

LeakSignal Proxy establishes a framework and delivery mechanism for composable inline traffic analysis and policy enforcement point within an existing sidecar.

The Proxy is written in Rust, and compiles to WASM (for Proxy-Wasm/Envoy) or Natively (Proxy-Wasm/Envoy/NGINX).

<p align="center">
  <img style={{"max-width": "75%"}} src="https://github.com/leaksignal/leaksignal/raw/master/assets/filter-overview2.png" />
</p>

The following functionality can be enabled through the Policy:

* Sensitive Data Observability
* Data Access by IP, Token, and Service
* Exfiltration Mitigation
* Data Access Auditing
* Dashboard visualization (histogram, heatmap)
* A powerful rules engine for alerts, blocks, and distributed ratelimits

### LeakSignal COMMAND

LeakSignal Command dashboard provides visibility and alerting when abnormal or unauthorized data access occurs. It's available publically as a SAAS offering, or on-prem for our enterprise customers.
<p align="center">
  <img style={{"max-width": "75%"}} src="https://github.com/leaksignal/leaksignal/raw/master/assets/command-overview1.png" />
</p>

### Architecture

LeakSignal Proxy handles inline traffic analysis and acts as a policy enforcement point for all Layer 4 and Layer 7 traffic. This includes, HTTP, Log Collection, Databases, etc.

The Proxy receives its policy from and reports its telemetry to LeakAgent or LeakSignal Command.

LeakAgent is a publicly-available Prometheus metrics adapter for LeakSignal telemetry that is free to run on-prem.

LeakSignal can be setup in the following modes:
* All metrics and configuration stay local in your environment. LeakAgent and Command can be hosted on-prem.
* All metrics and configuration live in the cloud, and telemetry is sent to the LeakSignal Command SAAS.

### Test and configure L7 Policy

After you've verified that the filter is running, you can configure the policy to check for specific sensitive data types or patterns. For examples of preconfigured and performance tested policies, see [LeakSignal Policies](Policy)

## Commercial support

- Leaksignal, Inc offers support and self-hosted versions of the cloud dashboard. Contact <sales@leaksignal.com>.

## License

Copyright 2024 LeakSignal, Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
