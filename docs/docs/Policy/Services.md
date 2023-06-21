---
sidebar_position: 4
---

# Services

Service configuration allows matching a group of services and applying specific policy to them.

## Format

Each service policy block has the following fields:

* `services`: One or more [Match Rule](Match%20Rule) that match a service name

## Example

```yaml
services:
  - services:
      - cluster.local/default/adservice
      - cluster.local/default/recommendationservice
      - cluster.local/default/checkoutservice
  - services:
      - cluster.local/default/shippingservice
      - cluster.local/default/currencyservice
      - cluster.local/default/cartservice
  - services:
      - cluster.local/default/paymentservice
      - cluster.local/default/emailservice
  - services: cluster.local/default/productcatalogservice
```
