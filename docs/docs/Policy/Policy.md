---
sidebar_position: 1
---

# Policy

LeakSignal Policies (LS policies or policies) are YAML configuration files that drive how LeakSignal scans for sensitive data.
There are two primary sections of LS policies, `categories` of matchers and `endpoints`.

This is an example of what an extremely complex policy might look like:

```yaml
categories:
  ssn:
    - regex: "\\b\\d{3}[ .-]\\d{2}[ .-]\\d{4}\\b"
  credit_card:
    - regex: "(?:4[0-9]{12}(?:[0-9]{3})?|[25][1-7][0-9]{14}|6(?:011|5[0-9][0-9])[0-9]{12}|3[47][0-9]{13}|3(?:0[0-5]|[68][0-9])[0-9]{11}|(?:2131|1800|35\\d{3})\\d{11})"
    - except: 0000-0000-0000-0000
  email:
    - regex: "\\b[a-zA-Z0-9][a-zA-Z0-9_.+-]{0,}@[a-zA-Z0-9][a-zA-Z0-9-.]{0,}\\.[a-zA-Z]{2,}\\b"
    - except: someone@example.com
  phone_number:
    - regex: "(?u)\\b(1[ .-]?)?[2-9]\\d{2}[ .-]?\\d{3}[ .-]?\\d{4}\\b|(\\b1[ .-]?)?\\([2-9]\\d{2}\\)[ .-]?\\d{3}[ .-]?\\d{4}\\b"
  address:
    - regex: "(?i)\\b\\d{1,5} ([a-zA-Z0-9]+ ){0,5}(street|st|avenue|ave|boulevard|blvd|road|rd|route|rt|rte|sr|way|wy|highway|hwy|camino|broadway|parkway|pkwy|expressway|expy?|drive|dr|lane|ln|trail|alley|loop|court|circle|park|plaza|place|square|terrace|row|walk|cove|cv|run|path|pass|hollow|bend),?( [a-zA-Z0-9#,\\.]+){0,3}\\b"
  phone_number_correlate:
    - regex: "\\b[0-9]{10}\\b"
    - correlate:
        interest: primary
        max_distance: 64
        matches:
          - raw_insensitive: phone
  keywords:
    - raw: "ssn"
    - raw_insensitive: "phone"
  any_1:
    - regex: ".*"
  any_2:
    - regex: ".*"
services:
  - services: "regex: .*"
sbac:
  - name: test_html
    stage: on_request_header_chunk
    filter:
      endpoint: /test.html
  - name: routing_matches
    stage: on_response_body_chunk
    filter:
      response_matches:
        routing: 2
  - name: json_matches
    stage: on_response_body_chunk
    filter:
      all:
        - endpoint: /test.json
        - response_matches:
            ssn: 10
  - name: json_path_false
    stage: on_response_body_chunk
    filter:
      response_matches:
        "*":
          - path: test.*[*]
          - exclude_path: test.my_ssn3.[*]
          - count: 2
  - name: json_path_true
    stage: on_response_body_chunk
    filter:
      response_matches:
        ssn:
          - path: test.*[*]
          - count: 2
ratelimits:
  - grouping: per_inbound_service
    by: service
    filter:
      peer_service:
        any:
        - sa: cartservice
        - sa: checkoutservice
    action: block
    timespan_secs: 30
    limit: 3000
  - grouping: per_endpoint
    by: ip
    filter:
      all:
        - endpoint: "**/api/**"
        - exclude_endpoint: "**/api/v1/health"
    action: alert
    timespan_secs: 60
    limit: 750
  - grouping: per_endpoint
    by: ip
    filter:
      all:
        - endpoint: "**/api/**"
        - exclude_endpoint: "**/api/v1/health"
        - exclude_token: exampleToken123
    action: block
    timespan_secs: 60
    limit: 1500
endpoints:
  - matches: "**"
    config:
      ssn:
        action: !redact "REDACTED"
      credit_card:
        content_types: json
      email:
        content_types: html
      phone_number: {}
      address: {}
      phone_number_correlate: {}
      keywords: {}
      any_1:
        action: !redact ""
        content_types: json
        contexts:
          - header: "content-type"
        search: response_header
      any_2:
        contexts:
          - header: "hello"
        search: request_header
    exploit_detection: {}
    token_extractor:
      location: request
      header: "block_me"
      regex: ".*"
      hash: true
blocked_tokens:
  - "9FC2BAB31B8C9738DAE5FEFBFF80B214011A7A317B75FC1AB352D17343F1166E"
blocked_ips:
  - "124.201.111.27"
report_style: raw
rules:
  - grouping: global
    by: ip
    filter:
      all:
        - response_outbound
        - response_matches:
            email: 1
            phone_number: 1
    name: "Potential Login Leak"
    severity: immediate
    action: alert
    timespan_secs: 30
    limit: 1
  - grouping: per_inbound_service
    by: outbound_service
    filter:
      all:
        - response_outbound
        - response_matches:
            snn: 1
            credit_card: 1
    name: "High Sensitive Data Access"
    severity: notable
    action: alert
    timespan_secs: 300
    limit: 10
body_collection:
  - mode: response_only
    sample_rate: 1.0
    filter:
      all:
      - response_outbound
      - response_matches:
          ssn: 20
          phone_number: 10
  - sample_rate: 1.0
    max_body_collection_mb: 0.05
    filter:
      any:
        - endpoint: "/form.html"
        - endpoint: "/bigjson.html"
  - mode: request_only
    sample_rate: 1.0
    filter:
      request_matches:
          ssn: 10
          phone_number: 10
header_collection: "all_request"
content_types:
  "application/imaginary": text
  "text/plain": none
```
