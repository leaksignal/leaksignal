---
sidebar_position: 2
---

# SBAC: Service Based Access Control

SBAC allows you to dynamically control the blocking of services based on a set of FilterRules

## Format

Each SBAC item has the following fields:

* `name`: Optional human readable name that will be displayed when the rule is responsible for blocking a service.
    If no rule is defined then the items index will be used instead.
* `stage`: The stage at which the rule should start being checked.
    The rule will only start being checked once the stage is hit and will be re-checked at every following stage. The possible values are:
  * `pre_request_headers`
  * `on_request_header_chunk`
  * `post_request_headers`
  * `on_request_body_chunk`
  * `post_request_body`
  * `on_response_header_chunk`
  * `post_response_headers`
  * `on_response_body_chunk`
  * `post_response_body`
  * `on_response_trailer_chunk`
  * `post_response_trailers`
* `filter`: the `RuleFilter` to check

## Example

```yaml
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
  - name: never
    stage: on_request_header_chunk
    filter:
      endpoint: /this/path/does/not/exist.html
  - name: json_matches
    stage: on_response_body_chunk
    filter:
      all:
        - endpoint: /test.json
        - response_matches:
            ssn: 10
  - name: header_matches_false
    stage: on_request_header_chunk
    filter:
      exclude_request_headers:
        hi: mars
        hello: world
```
