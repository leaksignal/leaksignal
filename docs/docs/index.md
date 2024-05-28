---
sidebar_position: 1
title: Getting Started
---

# Getting Started with LeakSignal

LeakSignal provides real-time visibility and governance over sensitive data flows within your services. As an openly distributed project, it offers comprehensive observability metrics and mitigation capabilities across the full spectrum of Layer 4 and 7 protocols.

## Features

- **Real-Time Layer 4 and 7 Analysis**  
  Inline inspection and classification of request and response data. Ideal for service mesh environments (OpenShift, Istio, Kubernetes) and WASM runtimes.

- **Flexible Policy Configuration**  
  Create and manage custom rules for detecting sensitive data leakage (e.g., PII, financial data).

- **Comprehensive Data Visibility**  
  Gain insights into data flows, including IP, token, and service-level access patterns. Provide visibility into data access in APIs, GraphQL, or gRPC streams (east/west and north/south). Map digital identities (non-human, human, etc.) to the data accessed.

- **Advanced Threat Mitigation**  
  Proactively identify and block unauthorized data access using real-time data classification.

- **Sensitive Data in Logs**  
  Observe and redact sensitive data in log streams before storage.

- **Egress Visibility**  
  Understand when sensitive data is sent out as part of a third-party call.

- **Robust Dashboard and Alerts**  
  Cloud-based dashboard with policy editor, monitoring, and alerting. Available for free as SaaS or self-hosted on-premises for enterprise customers.

- **Extensive Integration Support**  
  Metrics can be exposed via Envoy and integrated with OpenTelemetry, Prometheus, and other tools.

- **Scalable and Flexible Deployment**  
  Compatible with various environments, including Istio, OpenShift Service Mesh, NGINX, Lambda, Fastly Compute and more.

- **Enhanced Accuracy with Partner Integrations**  
  Leverage partner integrations (Cyera, BigID, etc) for tagging and classification to ensure there are little to no false positives.

## Installation

LeakSignal can deploy in a variety of environments -- Istio, OpenShift Service Mesh, NGINX/NGINX Ingress, Lambda, and more.

See detailed installation instructions [here](./Deployment/Istio%20&%20OSSM).

### Install LeakSignal from Your Preferred Marketplace

[Install LeakSignal on AWS](https://aws.amazon.com/marketplace/pp/prodview-4et32qmmt3yse)

[Install LeakSignal on Azure](https://azuremarketplace.microsoft.com/en-us/marketplace/apps/leaksignalinc1673983004536.leaksignal_test?tab=Overview)

[Install LeakSignal on Red Hat OpenShift](https://catalog.redhat.com/software/containers/leaksignal/leaksignal-operator/65bba2dfc5a5071d0ac06f82?architecture=amd64&image=65d63f73a90f7d622e03f5fd)

## Overview

LeakSignal detects sensitive data within mesh traffic. Analysis and processing of traffic are handled inline, allowing engineers to understand sensitive data emission without sending, storing, or viewing the sensitive data.

![Mesh Overview](../static/img/data_flow_across_services.png)

### LeakSignal Proxy

LeakSignal Proxy establishes a framework and delivery mechanism for composable inline traffic analysis and policy enforcement within an existing sidecar.

The Proxy is written in Rust and compiles to WASM (for Proxy-Wasm/Envoy) or Natively (Proxy-Wasm/Envoy/NGINX).

![Filter Overview](../static/img/full_solution_latest.png)

### LeakSignal COMMAND

The LeakSignal Command dashboard provides real-time visibility and alerting for unauthorized data access. It's available publicly as a SaaS or on-prem solution for enterprise customers.

![Command Overview](../static/img/ServiceMap_shorter.gif)

#### Key Functionalities:

- **Sensitive Data Observability**: Monitor and analyze sensitive data flows.
- **Data Access Management**: Track access by IP, token, and service.
- **Exfiltration Mitigation**: Prevent unauthorized data transfers.
- **Audit Trails**: Maintain logs of data access and actions.
- **Dashboard Visualization**: View data through histograms and heatmaps.
- **Rules Engine**: Set up alerts, blocks, and distributed rate limits.

### Architecture

LeakSignal Proxy performs inline traffic analysis and acts as a policy enforcement point for all Layer 4 and Layer 7 traffic, including HTTP, log collection, and databases. Policies and telemetry are managed through LeakAgent or LeakSignal Command.

- **LeakAgent**: A free, on-prem Prometheus metrics adapter.
- **Command SaaS**: Free, Cloud-hosted metrics and configuration.
- **Command On-Prem**: The self-hosted commercial offering.

### Setup Modes:

- **Local Setup**: All metrics and configuration remain on-prem.
- **Cloud Setup**: Metrics and configuration are managed in the cloud with telemetry sent to the LeakSignal Command SaaS.

## Commercial Support

LeakSignal, Inc offers support and self-hosted versions of the cloud dashboard. Contact us at <sales@leaksignal.com>.

## License

Copyright 2024 LeakSignal, Inc.

Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at

http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.