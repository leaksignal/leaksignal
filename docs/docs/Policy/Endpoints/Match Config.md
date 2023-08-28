---
sidebar_position: 3
---

# Match Config

## Format

All fields in a Match Config are optional and have defaults. For straightforward matching uses, an empty object (`{}`) can often suffice.

### matcher

By default the Match Config will use the matcher corresponding to the category name. This field allows you to override that and instead use a matcher of your choosing

```yaml
any_1:
  matcher: any
  action: !redact ""
  content_types: json
  contexts: !header "content-type"
  search: response_header
any_2:
  matcher: any
  contexts: !header "hello"
  search: request_header
```

### name

By default the Match Config will emit matches with the name of the category. This field allows you to override that and instead use a name of your choosing

```yaml
any:
  action: !redact ""
  content_types: json
  contexts: !header "content-type"
  search: response_header
any_2:
  name: any
  matcher: any
  contexts: !header "hello"
  search: request_header
```

### action

The `action` field specifies how a match is to be reacted to. The main use of this is to disable a rule temporarily or halt a response body for some particularly sensitive data.

#### Valid Action Values

* `ignore`: Do nothing. Effectively deletes this configuration.
* `alert`: Forwards match data upstream, not to be confused with alerting. This is the default.
* `block`: If found, a response is aborted and no further data is written to the network. This is messy and isn't recommended except in exceptional situations.
* `redact`: Redacts the match with a user provided mask. Unicode in mask not supported and will throw an error during policy deserialization.

### content_types

The `content_types` field can be used to set a whitelist on content types.
The default is no whitelist at all (all content types are allowed).

#### Valid content_type Values

Can be one or more of the following values, as a string or array of strings.

* `html`
* `json`

### contexts

The `contexts` field can be used to set a whitelist on a subset of a response.
The default is no whitelist at all (all contexts are allowed).

Interpretation of `contexts` depends on the content type of the document.

#### `json` Valid Values

Can be one or more of the following values, as a string or array of strings.

* `keys`: Matches on keys only
* `values`: Matches on values only
* `values_key`: Matches on the keys of each value and emits the value if the key matches

### report_style

Same as [report_style in Endpoint Config](Endpoint%20Config#report_style)

### report_bits

Same as [report_bits in Endpoint Config](Endpoint%20Config#report_bits)

## Examples

### Match specific json keys

```yaml
categories:
  name_key:
    - raw: name
    - raw: first_name
    - raw: last_name
endpoints:
  - matches: "**"
    config:
      name_key:
        content_types: json
        contexts: keys
```

### Report & alert for credit cards

```yaml
categories:
  credit_card:
    - regex: "\\d{4}[\\s.-]\\d{4}[\\s.-]\\d{4}[\\s.-]\\d{4}"
    - regex: "(?:4[0-9]{12}(?:[0-9]{3})?|[25][1-7][0-9]{14}|6(?:011|5[0-9][0-9])[0-9]{12}|3[47][0-9]{13}|3(?:0[0-5]|[68][0-9])[0-9]{11}|(?:2131|1800|35\\d{3})\\d{11})"
    - except: 0000-0000-0000-0000
endpoints:
  - matches: "**"
    config:
      credit_card:
        report_style: partial_sha256
        report_bits: 32
```
