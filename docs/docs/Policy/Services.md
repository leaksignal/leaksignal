---
sidebar_position: 4
---
Service configuration allows matching a group of services and applying specific policy to them.

## Format

Each service policy block has the following fields:

* `services`: One or more [Match Rule](Match%20Rule) that match a service name
* `blacklist`: Optionally specified, one or more [Match Rules](Match%20Rule) that match on services, of which are blacklisted from communicating with services in this policy. Ignored if `whitelist` is non-empty
* `whitelist`: Optionally specified, one or more [Match Rules](Match%20Rule) that match on services. If none of these rules match an inbound service, it's communication is blocked
* `block_unknown_services`: If `whitelist` is nonempty, this defaults to `true`. Otherwise, `false`. When `true`, inbound communications from unknown services (no mTLS) is blocked

## Example

```
services:
  - services:
      - cluster.local/default/adservice
      - cluster.local/default/recommendationservice
      - cluster.local/default/checkoutservice
    whitelist:
      - cluster.local/default/frontend
  - services:
      - cluster.local/default/shippingservice
      - cluster.local/default/currencyservice
      - cluster.local/default/cartservice
    whitelist:
      - cluster.local/default/frontend
      - cluster.local/default/checkoutservice
  - services:
      - cluster.local/default/paymentservice
      - cluster.local/default/emailservice
    whitelist:
      - cluster.local/default/checkoutservice
  - services: cluster.local/default/productcatalogservice
    whitelist:
      - cluster.local/default/frontend
      - cluster.local/default/recommendationservice
      - cluster.local/default/checkoutservice
```
