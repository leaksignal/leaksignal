---
sidebar_position: 3
---

# Rules

Rules allow specific endpoints, services, and clusters to ratelimit (timed block) or alert on individual ips, tokens, or service names.

## Format

Rules are specified in the top-level `rules` field of the policy.

Fields:

* `grouping`: `per_inbound_service`, `per_outbound_service`, `per_endpoint`, or `global` -- specifies the grouping of inbound data for rule execution. I.e. a `per_inbound_service` rule by `ip` looks at the rate of traffic of each ip for each service, and evaluates the rule.
* `by`: `ip` (default), `token`, `service`, or an object with a single field `header` signifying a header name -- the unique ID to be used for the rule. Note that response-derived tokens will still let the initial request through.
* `count_by`: Same as `by`, optional. If set, only unique values for this field will increment the counter in a given timespan.
* `action`: `block` (default), `alert`, or `nothing` -- the action to take upon the rule conditions being met.
* `timespan_secs`: The timespan over which requests/responses are counted for rule evaluation. This effectively means that a client is limited to `limit`/`timespan_secs` requests per second.
* `limit`: The maximum number of requests within `timespan_secs` before the rule is triggered
* `filter`: The evaluation filter of the rule, see below.

### Filters

Each filter has exactly one of the following keys:

* `endpoint`: Takes one of more [Path Globs](Endpoints/Path%20Globs) and requires that **any** of them match the policy path
* `exclude_endpoint`: Takes one of more [Path Globs](Endpoints/Path%20Globs) and requires that **all** of them **do not** match the policy path
* `peer_service`: Takes one or more [Match Rule](Match%20Rule) and requires that **any** of them match the peer_service
* `exclude_peer_service`: Takes one or more [Match Rule](Match%20Rule) and requires that **all** of them **do not** match the peer_service
* `local_service`: Takes one or more [Match Rule](Match%20Rule) and requires that **any** of them match the local_service
* `exclude_local_service`: Takes one or more [Match Rule](Match%20Rule) and requires that **all** of them **do not** match the local_service
* `token`: Takes one or more [Match Rule](Match%20Rule) and requires that **any** of them match the token. A missing token is an empty string
* `exclude_token`: Takes one or more [Match Rule](Match%20Rule) and requires that **all** of them **do not** match the token. A missing token is an empty string
* `ip`: Takes one or more IP address or CIDR and requires that **any** of them match the ip
* `exclude_ip`: Takes one or more IP address or CIDR and requires that **all** of them **do not** match the ip
* `request_matches`: Takes a map of category names and one or more [Match Rule Filters](#Match%20Rule%20Filter) that the category must satisfy. Use "*" to match any category.
* `response_matches`:  Takes a map of category names and one or more [Match Rule Filters](#Match%20Rule%20Filter) that the category must satisfy. Use "*" to match any category.
* `request_headers`: Takes a map of header names and one or more [Match Rule](Match%20Rule) and, for each header listed, requires that the header key exist and **any** of the rules match the header value
* `exclude_request_headers`: Takes a map of header names and one or more [Match Rule](Match%20Rule) and, for each header listed, requires that the header key either doesn't exist or that **none of** of the rules match the header value
* `response_headers`: Takes a map of header names and one or more [Match Rule](Match%20Rule) and, for each header listed, requires that the header key exist and **any** of the rules match the header value
* `exclude_response_headers`: Takes a map of header names and one or more [Match Rule](Match%20Rule) and, for each header listed, requires that the header key either doesn't exist or that **none of** of the rules match the header value
* `response_trailers`: Takes a map of trailer names and one or more [Match Rule](Match%20Rule) and, for each trailer listed, requires that the trailer key exist and **any** of the rules match the trailer value
* `exclude_response_trailers`: Takes a map of trailer names and one or more [Match Rule](Match%20Rule) and, for each trailer listed, requires that the trailer key either doesn't exist or that **none of** of the rules match the trailer value
* `request_cookie`: Takes a map of names and one or more [Match Rule](Match%20Rule) and, for each key listed, requires that the request cookie header contains the key and **any** of the rules match the header value
* `exclude_request_cookie`: Takes a map of names and one or more [Match Rule](Match%20Rule) and, for each key listed, requires that the request cookie either doesn't contain the key or that **none of** of the rules match the key's value
* `response_outbound`: Matches request/response pairs for which the request is inbound and the response is outbound
* `response_inbound`: Matches request/response pairs for which the request is outbound and the response is inbound
* `any`: Takes one or more Filters and requires that **any** of them match the request/response
* `all`: Takes one or more Filters and requires that **all** of them match the request/response

#### Match Rule Filter

* `path`: A json path that the match belongs to. Can use wildcards like `test.my_ssn.*.ssn[*]`.
* `exclude_path`: A json path that the match must not belong to. Can use wildcards like `test.my_ssn.*.ssn[*]`.
* `count`: The minimum amount of matches to qualify. Defaults to `1` if not specified

## Examples

```yaml
rules:
  - grouping: per_inbound_service
    by: service
    filter:
      any:
        - peer_service: "cluster.local/default/cartservice"
        - peer_service: "cluster.local/default/checkoutservice"
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
  - grouping: global
    by: ip
    timespan_secs: 10
    limit: 10
    filter:
      peer_service: external
```
