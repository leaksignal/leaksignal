---
sidebar_position: 4
---

# Services

Service configuration allows matching a group of services and applying specific policy to them.

## Format

Each service policy block has the following fields:

* `filter`: A [Service Filters](#Service%20Filter) to select which services this policy applies to.
* `endpoints`: Endpoint configuration for just services selected by the `filter`
* `use_default_endpoints`: If `true` (default), then the global endpoints configuration is also used for matching services at a lower priority.


## Service Filter

Each service filter has exactly one of the following keys:

* `cluster`: Takes one or more [Match Rule](Match%20Rule) and requires that **any** of them match the cluster component
* `exclude_cluster`: Takes one or more [Match Rule](Match%20Rule) and requires that **all** of them **do not** match the cluster component
* `ns`: Takes one or more [Match Rule](Match%20Rule) and requires that **any** of them match the namespace component
* `exclude_ns`: Takes one or more [Match Rule](Match%20Rule) and requires that **all** of them **do not** match the namespace component
* `sa`: Takes one or more [Match Rule](Match%20Rule) and requires that **any** of them match the service account component
* `exclude_sa`: Takes one or more [Match Rule](Match%20Rule) and requires that **all** of them **do not** match the service account component
* `workload`: Takes one or more [Match Rule](Match%20Rule) and requires that **any** of them match the workload component
* `exclude_workload`: Takes one or more [Match Rule](Match%20Rule) and requires that **all** of them **do not** match the workload component
* `response_outbound`: Matches request/response pairs for which the request is inbound and the response is outbound
* `response_inbound`: Matches request/response pairs for which the request is outbound and the response is inbound
* `any`: Takes one or more Service Filters and requires that **any** of them match the request/response
* `all`: Takes one or more Service Filters and requires that **all** of them match the request/response

## Example

```yaml
services:
  - filter:
      ns: marketing
    endpoints:
    - matches: "**"
      config:
        phone_number: {}
```
