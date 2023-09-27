---
sidebar_position: 2
---

## Deployment with a Helm Chart

LeakAgent can be deployed with a Helm chart.

The Helm Chart lives in the LeakSignal repository [here](https://github.com/leaksignal/leaksignal/tree/master/leakagent_helm).

See [Examples](https://github.com/leaksignal/leaksignal/tree/master/examples/leakagent_helm).

### Helm Install Command
```
helm upgrade --install leakagent oci://registry-1.docker.io/leaksignal/leakagent \
  --version 0.7.0-helm \
  --namespace leakagent --create-namespace

# or more generally

helm upgrade --install leakagent oci://registry-1.docker.io/leaksignal/leakagent \
  --version <version>-helm \
  --namespace leakagent --create-namespace

# with a specific values.yaml (i.e. to set policies)

helm upgrade --install leakagent oci://registry-1.docker.io/leaksignal/leakagent \
  --version 0.7.0-helm \
  -f ./values.yaml \
  --namespace leakagent --create-namespace
```
