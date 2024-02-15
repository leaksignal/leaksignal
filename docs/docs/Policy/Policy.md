---
sidebar_position: 3
---


## Major Components

* `categories` which define what kind of sensitive data we are looking for
* `endpoints` which define where to look for that sensitive data
* `services` which define service-specific endpoint configuration, in addition or instead of the default `endpoints`
* `sbac` which define hard blocking rules (i.e. A credit card number is never dispatched to the outside internet)
* `rules` which define distributed alerts and ratelimits (i.e. Non-admin users cannot see more than 10 distinct credit cards in an hour)
* `body_collection` which define when to upload entire request/response/stream bodies
* `content_types`
* `stream_types` `stream_upload_interval`

## Misc Components

* `collected_request_headers`: Specifies a list of HTTP request headers to send upstream for telemetry. Overrides the default list if set.
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
* `collected_response_headers`: Specifies a list of HTTP response headers to send upstream for telemetry. Overrides the default list if set.
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
* `header_collection`: An override for 
* `report_style`
* `report_bits`
* `local_service_name` ServiceNameExtractor
* `peer_service_name`
* `path_groups`
