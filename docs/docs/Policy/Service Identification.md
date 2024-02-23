---
sidebar_position: 5
---

There are two fields at the root level of the policy, `local_service_name` and `peer_service_name` that specify service identification. They default to the best option for Istio.

These fields specify how we identify the service we are running on (`local_service_name`), and the service we are talking to (`peer_service_name`).

Each service identification configuration has the following format:
```yaml
# any missing fields are filled in with defaults

# The name of the cluster the workload is on
cluster: Source,
# The namespace of the workload
ns: Source,
# The service account or service name of the workload
sa: Source,
# The workload name
workload: Source,

# where Source is one of:

# Returns nothing
"none"
# Uses the Istio SPIFFE ID
"istio"
# Uses an environment variable
!env "ENV_VAR"
# Uses a connection attribute
!attrs "ATTR_NAME"
# Uses a static value
!raw "STATIC_VALUE"
# select the name from `from` if present, otherwise `to`.
default:
    from: Source
    to: Source
```
