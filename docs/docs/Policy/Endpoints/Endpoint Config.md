---
sidebar_position: 2
---

# Endpoint Config

An individual endpoint block is composed of one or more of path globs for matching paths, followed by a configuration set for the different matching rules, and two optional configuration sets for token extraction and data reporting style.

## Format

### matches

Each endpoint definition takes one or more [Rule Filters](../Rules#Filters) in the `matches` field that define the targets that this endpoint applies for.

If a given request target matches multiple endpoints, the rules of those endpoints are ordered most general to least general. Any configurations of those more specific rules selectively override the more general configurations.

### config

The `config` field is a mapping from category name to a specific [Match Config](Match%20Config). An empty object is frequently enough for a given configuration.

### token_extractor

The `token_extractor` field is a [Token Extraction](Token%20Extraction) object that specifies how request/response tokens are captured and processed.

#### location

The `location` field tells LeakSignal where to look for the token.

Valid Values:

* `request`: Pulls from request headers
* `request_cookie`: Pulls from a specific request cookie
* `reponse`: Pulls from response headers. This option is ideal as it is not vulnerable to client-side forgery.

#### header

The `header` field specifies the header or cookie name from which to extract the token.

#### regex

The `regex` field optionally specifies a regex to validate a token. If there is a first capture group, it is returned; otherwise, the entire regex match is returned. If no `regex` is specified, the entire token is returned.

#### hash

The `hash` field is a boolean toggle, when set the token is SHA-256 hashed. Should be used for tokens that are vulnerable to malicious reuse. Defaults to false.

### report_style

The `report_style` field specifies the general report style for requests that match this endpoint configuration. Can be overridden by individual [Match Config](Match%20Config#report_style) in `config`, and overrides `report_style` at the root-level of the policy.

#### Valid Values

* `raw`: the raw matched value is reported
* `partial_sha256`: the first `report_bits` bits of the SHA-256 hash of the matched value is reported
* `sha256`: The complete SHA-256 hash of the matched value is reported
* `none`: No data is reported. Alerts in LeakSignal Command will not be able to deduplicate values matched many times.

### report_bits

The `report_bits` field is only used when `report_style` is `partial_sha256`. It specifies how many leading bits of the SHA-256 hash to retain.

## Examples

### Match rules for all endpoints

```yaml
endpoints:
  - matches: "**"
    config:
      email: {}
      phone_number: {}
      address: {}
      date_of_birth: {}
```

### Common match rules for some specific endpoints

```yaml
endpoints:
  - matches:
    - */api/v1/email
    - */api/v1/profile
    config:
      email: {}
```

### Extracting a JWT for a token

This example extracts a JWT from a request `Authorization` header and also strips the signature to prevent malicious token reuse.

```yaml
endpoints:
  - matches: "**"
    token_extractor:
      location: request
      header: Authorization
      regex: "Token ([^\\.]+\\.[^\\.]+)\\.[^\\.]+"
    config:
      email: {}
```

### Partial SHA-256 reporting

```yaml
endpoints:
  - matches: "**"
    config:
      credit_card: {}
    report_style: partial_sha256
    report_bits: 32
```

### Comprehensive Example

```yaml
endpoints:
  - matches: "**"
    config:
      name_key:
        content_types: json
        contexts: keys
      credit_card:
        report_style: partial_sha256
        report_bits: 32
      ssn:
        report_style: partial_sha256
        report_bits: 24
      email: {}
      phone_number: {}
      address: {}
      date_of_birth: {}
      phone_number_correlate: {}
```
