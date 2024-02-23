---
sidebar_position: 6
---

# Other Config Options

## collected_request_headers

The `collected_request_headers` field specifies request headers that are not redacted. Default list is:

- :path
- :method
- :authority
- :scheme
- host
- accept
- accept-encoding
- accept-language
- cache-control
- referer
- user-agent
- x-request-id
- x-forwarded-for
- content-type
- grpc-encoding
- grpc-accept-encoding
- x-envoy-peer-metadata-id

## collected_response_headers

The `collected_response_headers` field specifies request headers that are not redacted. Default list is:

- :status
- content-encoding
- content-type
- date
- server
- vary
- via
- grpc-encoding
- grpc-accept-encoding
- x-envoy-peer-metadata-id
- grpc-status
- grpc-message
- x-ls-request-id
- x-source
- x-ls-source
- x-sbac-rule

## header_collection

When to collect headers. Possible values are:

- `all_request`
- `all_response`
- `all`
- `none` (default)

## report_style

Same as [report_style in Endpoint Config](Endpoints/Endpoint%20Config#report_style)

## report_bits

Same as [report_bits in Endpoint Config](Endpoints/Endpoint%20Config#report_bits)

## content_types

Defines which parsers to use on which content types.

The possible parsers are:

- `text`: parse as plaintext
- `json`: parse as json keys/values
- `grpc`: parse as a GRPC stream
- `filebeat`: parse as a filebeat stream (streaming mode only)
- `tls`: parse as a TLS stream (streaming mode only). this doesn't perform any actual matching, and instead adds the field `ls_tls` to the connection info with a boolean for if it is a TLS stream or not. If the parser hasn't seen enough bytes to determine if the stream is TLS or not, then `ls_tls` will contain `unknown`, although this should basically never happen as it only takes a handful of bytes to determine this
- `none`: do not parse this content type (default)

The default types are:

- `text/html`: text
- `text/plain`: text
- `text/xml`: text
- `application/soap+xml`: text
- `application/atom+xml`: text
- `application/xhtml+xml`: text
- `application/vnd.mozilla.xul+xml`: text
- `application/xml`: text
- `application/vnd.openxmlformats-officedocument.spreadsheetml.sheet`: text
- `application/vnd.openxmlformats-officedocument.presentationml.presentation`: text
- `application/grpc`: grpc
- `application/grpc+proto`: grpc
- `application/json`: json
- `application/ld+json`: json

Associations for custom content types or overriding of the defaults would look like the following:

```yaml
content_types:
  "application/imaginary": text
  "text/plain": none
```

## path_groups

An optional set of [Path Globs](Endpoints/Path%20Globs) that can be used to group similar policy paths together.
Useful for preventing "path explosions".
If an endpoint doesnt match any of the provided globs, then the policy path will be set to the endpoint path with any numerical sections removed.
