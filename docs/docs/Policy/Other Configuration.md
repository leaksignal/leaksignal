---
sidebar_position: 6
---

## collected_request_headers
The `collected_request_headers` field specifies request headers that are not redacted. Default list is available at [`leakpolicy/src/lib.rs`](https://github.com/leaksignal/leaksignal/blob/master/leakpolicy/src/lib.rs#L279).

## collected_response_headers
The `collected_response_headers` field specifies request headers that are not redacted. Default list is available at [`leakpolicy/src/lib.rs`](https://github.com/leaksignal/leaksignal/blob/master/leakpolicy/src/lib.rs#L299).

## body_collection_rate
The `body_collection_rate` field is a floating point ratio between 0.0 and 1.0 denoting how often responses are to be recorded in their entirety __without redaction__. Defaults to 0.0. Responses are able to be retrived in the dashboard via a "Download Body" button.

## report_style
The `report_style` field is an object unlike [report_style in Endpoint Config](Endpoints/Endpoint%20Config#report_style).

Fields:
### report_style
Same as [report_style in Endpoint Config](Endpoints/Endpoint%20Config#report_style)

### report_bits
Same as [report_bits in Endpoint Config](Endpoints/Endpoint%20Config#report_bits)
