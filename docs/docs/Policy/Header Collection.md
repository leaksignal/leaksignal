---
sidebar_position: 9
---

LeakSignal defaults to collecting a whitelist of HTTP request and response headers.

### Fields

#### collected_request_headers
`collected_request_headers` specifies a list of HTTP request headers to send upstream for telemetry. Overrides the default list if set.
<details>
<summary>
  Default
</summary>
  - :path<br/>
  - :method<br/>
  - :authority<br/>
  - :scheme<br/>
  - host<br/>
  - accept<br/>
  - accept-encoding<br/>
  - accept-language<br/>
  - cache-control<br/>
  - referer<br/>
  - user-agent<br/>
  - x-request-id<br/>
  - x-forwarded-for<br/>
  - content-type<br/>
  - grpc-encoding<br/>
  - grpc-accept-encoding<br/>
  - x-envoy-peer-metadata-id<br/>
</details>

#### collected_response_headers
`collected_response_headers` specifies a list of HTTP response headers to send upstream for telemetry. Overrides the default list if set.
<details>
<summary>
  Default
</summary>
  - :status<br/>
  - content-encoding<br/>
  - content-type<br/>
  - date<br/>
  - server<br/>
  - vary<br/>
  - via<br/>
  - grpc-encoding<br/>
  - grpc-accept-encoding<br/>
  - x-envoy-peer-metadata-id<br/>
  - grpc-status<br/>
  - grpc-message<br/>
  - x-ls-request-id<br/>
  - x-source<br/>
  - x-ls-source<br/>
  - x-sbac-rule<br/>
</details>

#### header_collection

`header_collection` overrides `collected_request_headers`/`collected_response_headers`. Can be set to `all_request`, `all_response`, `all`, or `none` (default) to skip the whitelist.
