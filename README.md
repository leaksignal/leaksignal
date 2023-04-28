# LeakSignal [![Tweet](https://img.shields.io/twitter/url/http/shields.io.svg?style=social)](https://twitter.com/intent/tweet?text=Mesh%20Runtime%20Security.%20Protection%20across%20all%20Service%20mesh%20protocols%21&url=https://github.com/leaksignal/leaksignal/&hashtags=leaksignal,meshsecops)

**Mesh Native Runtime Security 🎉**

<a href="https://www.leaksignal.com"><p align="center">
  <img src="assets/logo-black-red.png?sanitize=true" width="800">
</p></a>

<h4 align="center">
  <a href="https://www.leaksignal.com">Website</a> |
  <a href="https://www.leaksignal.com/docs/">Docs</a> |
  <a href="https://www.leaksignal.com/blog/">Blog</a> | 
  <a href="https://join.slack.com/t/leaksignal-workspace/shared_invite/zt-1k98fc72o-qslbDyGZJeS638zDRvB3xw">Slack</a>
</h4>

<p align="center">
  <a href="https://github.com/leaksignal/leaksignal/blob/master/LICENSE"><img src="https://img.shields.io/hexpm/l/plug" alt="License"></a>
</p>

<p align="center">🔍 How can I observe and secure sensitive data travelling across the Service Mesh data plane without impacting performance? 🤷</p>

## 📙 Documentation

LeakSignal installation and reference documents are available at leaksignal.com.

👉 **[Quick Start](https://www.leaksignal.com/docs/#quickstarts)**

👉 **[Installation](https://www.leaksignal.com/docs/#getting-started-with-a-demo-application)**

👉 **[Sample Policies](https://github.com/leaksignal/leaksignal/tree/master/examples/policies)**

LeakSignal provides observability metrics and redaction capabilities for sensitive data contained within service mesh protocols. LeakSignal metrics can be consumed by Prometheus, pushed as OpenTelemetry, or collected in a centralized dashboard - giving MeshSecOps engineers (Incident Repsponse, SRE, DevOps, Platform Eng., SOC etc) a new security tool to help combat API exploits, unknown misconfigurations and sensitive data leakage.

## Features
* Fast, inline Layer 7 request/response analysis.
* Easy to configure rules ("L7 policy") for detecting and analyzing sensitive data (e.g. PII) leakage.
  * Detect PII, part numbers, account numbers, patient info, grades, dates, email addresses, large arrays, etc. You can write your own matcher or use our constantly evolving <a href="https://github.com/leaksignal/leaksignal/tree/master/examples/policies">ruleset</a> library (contributions welcome).
* Cloud dashboard with policy editor, monitoring, and alerting.
* Analysis metrics can be exposed via Envoy and thus reflected wherever Envoy metrics are configured to land (OpenTelemetry, Prometheus, etc.)


## Community / How to Contribute
* [Code contribution guidelines](/CONTRIBUTIONS.md)

## Commercial support
- Leaksignal, Inc offers [enterprise support](https://leaksignal.com) and self-hosted versions of the cloud dashboard. Contact sales@leaksignal.com.

## License 
Copyright 2023 LeakSignal, Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.


