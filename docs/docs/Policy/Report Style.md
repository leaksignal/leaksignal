---
sidebar_position: 10
---

Frequently, we do not want sensitive information being sent upstream to Command. To faciliate that, LeakSignal can be configured in several ways to limit or eliminate sensitive information being sent upstream.

## Fields

### report_style

The `report_style` field specifies the general report style for requests that match this endpoint configuration.

* `raw`: The raw matched value is reported.
* `partial_sha256`: the first `report_bits` bits of the SHA-256 hash of the matched value is reported. Expects an accompanying `report_bits` field.
* `sha256`: The complete SHA-256 hash of the matched value is reported.
* `none`: No data is reported. Alerts in LeakSignal Command will not be able to deduplicate values matched many times.

### report_bits

The `report_bits` field is only used when `report_style` is `partial_sha256`. It specifies how many leading bits of the SHA-256 hash to retain.


## Hierarchy

The default report style is defined at the root level of the policy, then can be overriden in [Categories](Categories) or [Endpoints](Endpoints/Overview).

## Choosing a Report Style

For data that is not sensitve and where the value is useful in analysis (i.e. a user ID), the default `raw` style is appropriate.

When dealing with sensitive information that has comparable or less bits of entropy than a SHA-256 hash (i.e. credit cards, SSNs), `partial_sha256` with an accompanying `report_bits` is recommended. The hash will be truncated before it's sent to Command.

When dealing with oversize amounts of data, or sensitive data that has sufficient entropy, `sha256` can be used.

In cases where no value reporting is required, `none` can be used. Command will not be able to de-duplicate values that are matched multiple times.
