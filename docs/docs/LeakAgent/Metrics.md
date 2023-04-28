---
sidebar_position: 2
---

## LeakAgent Metrics
These metrics show the internal state of LeakAgent, and don't pertain to per-transaction match information.

#### leakagent_match_inbound
Type: Counter
Fields:
* `deployment`

Description: Count of inbound transactions

#### leakagent_proxy_pull
Type: Counter
Fields:
* `key`

Description: Count of proxy pull requests

#### leakagent_policy_update_turnaway
Type: Counter
Fields:
* `deployment`
* `ip`

Description: Count of turned away nodes (invalid `api_key`/deployment name)

#### leakagent_policy_update_init
Type: Counter
Fields:
* `deployment`
* `ip`
* `policy`

Description: Count of policy update streams initialized

#### leakagent_policy_update_send
Type: Counter
Fields:
* `deployment`
* `policy`

Description: Count of dispatched policies

#### leakagent_policy_update_waiting
Type: Gauge
Fields:
* `deployment`

Description: Number of connections waiting for policy updates. Effectively the number of  currently connected proxies.

## LeakSignal Metrics
These metrics show per-transaction (request & response pair) details

#### leaksignal_status_code
Type: Counter
Fields:
* `deployment`
* `policy_path`
* `local_service`
* `peer_service`
* `is_outbound`
* `ip`
* `token`
* `status`

Description: HTTP status codes in responses.

#### leaksignal_match_time_ms
Type: [Conventional Histogram](https://prometheus.io/docs/practices/histograms/)
Fields:
* `deployment`
* `policy_path`
* `local_service`
* `peer_service`
* `is_outbound`
* `ip`
* `token`

Description: Time taken for all LeakSignal matchers to execute.

#### leaksignal_request_time_ms
Type: Conventional Histogram
Fields:
* `deployment`
* `policy_path`
* `local_service`
* `peer_service`
* `is_outbound`
* `ip`
* `token`

Description: Time taken from first request header to last of the request body. Includes all network time and processing time.

#### leaksignal_response_time_ms
Type: Conventional Histogram
Fields:
* `deployment`
* `policy_path`
* `local_service`
* `peer_service`
* `is_outbound`
* `ip`
* `token`

Description: Time taken from end of the request body to the end of the response body. Includes all network time and processing time.

#### leaksignal_request_size_bytes
Type: Conventional Histogram
Fields:
* `deployment`
* `policy_path`
* `local_service`
* `peer_service`
* `is_outbound`
* `ip`
* `token`

Description: Size in bytes of request bodies, or 0 if not present.

#### leaksignal_response_size_bytes
Type: Conventional Histogram
Fields:
* `deployment`
* `policy_path`
* `local_service`
* `peer_service`
* `is_outbound`
* `ip`
* `token`

Description: Size in bytes of response bodies.

## LeakSignal Match Metrics
These metrics are emitted for each match in a given transaction, meaning each transaction will have 0 or more associated events here.

#### leaksignal_match_count
Type: Conventional Histogram
Fields:
* `deployment`
* `policy_path`
* `local_service`
* `peer_service`
* `is_outbound`
* `ip`
* `token`
* `in_response`
* `in_header`
* `category`

Description: The match count of a given category in a given context.