---
sidebar_position: 8
---

The `body_collection` field contains a list of rules for when to emit the body of a request/response to command. bodies are recorded in their entirety __without redaction__. Submitted bodies are able to be retrieved in the dashboard via a "Download Body" button.
Each entry has the following internal fields:

## Fields

- `mode`: `request_only`, `downstream_only` (streaming only), `response_only`, `upstream_only` (streaming only), `all`. Defaults to `all`
- `sample_rate`: a floating point ratio between 0.0 and 1.0 denoting how often matching bodies are to be recorded
- `max_body_collection_mb`: a floating point determining the maximum size a collected body can be. defaults to 16.0 and will error during parse if below 0.0
- `filter`: an optional list of [Rule Filter](Rules#rule-filter) that the request/response must match to warrant body collection

## Example

```yaml
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
```
