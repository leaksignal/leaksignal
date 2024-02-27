---
sidebar_position: 1
---

## Requirements

* Kubernetes
* Istio or OpenShift Service Mesh

## OperatorHub/OLM

LeakSignal Operator is available on [OperatorHub](https://operatorhub.io/operator/leaksignal-operator), or as a [Certified Operator on OpenShift OperatorHub](https://catalog.redhat.com/software/container-stacks/detail/64f9f47e9c7ac3eb6ed9605d).

You can create a subscription to deploy the LeakSignal Operator:
```yaml
apiVersion: operators.coreos.com/v1alpha1
kind: Subscription
metadata:
  name: leaksignal-operator
  namespace: operators
spec:
  channel: stable
  name: leaksignal-operator
  source: operatorhubio-catalog
  sourceNamespace: olm
  installPlanApproval: Automatic
```

## Docker Images

There is a public image available:

* [`leaksignal/leaksignal-operator`](https://hub.docker.com/r/leaksignal/leaksignal-operator)

The version is not correlated with the LeakSignal version. Versions with `-ubi` suffixes are built on the RedHat UBI image. The `-helm` suffixed tags contain the Helm chart to deploy the operator.

## Helm Chart

The easiest way to deploy the Operator without OLM is with the Helm Chart:

```
helm upgrade --install leaksignal-operator oci://registry-1.docker.io/leaksignal/leaksignal-operator \
  --version 1.6.2-helm \
  --namespace leaksignal-operator \
  --create-namespace
```

If updating from a previous version via Helm, make sure to manually update the CRDs:
```bash
$ kubectl apply -f https://raw.githubusercontent.com/leaksignal/leaksignal-operator/v1.6.2/crds/leaksignal-crd.yaml https://raw.githubusercontent.com/leaksignal/leaksignal-operator/v1.6.2/crds/leaksignal-cluster-crd.yaml
```

### Helm Values

Check out our [GitHub Repository](https://github.com/leaksignal/leaksignal-operator/tree/master/chart) for details on Helm configurable values.

For most situations, the defaults will work right out of the box.

### Next Steps

Check out [Getting Started](./Getting%20Started) to get LeakSignal deployed!