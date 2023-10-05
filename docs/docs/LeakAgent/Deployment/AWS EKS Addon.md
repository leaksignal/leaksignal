---
sidebar_position: 3
---

## Prerequisites

* Subscription to [LeakAgent by LeakSignal](https://aws.amazon.com/marketplace/pp/prodview-4et32qmmt3yse)
* An existing EKS cluster running Kubernetes 1.24+
* Visit the [AWS License Manager](https://us-west-2.console.aws.amazon.com/license-manager/home) in AWS console for the first time to setup the default role to consume licenses.

## Installing LeakAgent using the AWS Console

1. Select your desired cluster in the [AWS Elastic Kubernetes Service Console](https://us-west-2.console.aws.amazon.com/eks/home?region=us-west-2#/clusters)
2. Select the "Add-ons" tab and click "Get more add-ons" to search AWS Marketplace add-ons for LeakAgent by LeakSignal
3. Under Select IAM role, select "Inherit from node"
4. Follow the onscreen instructions to complete deployment of LeakAgent add-on

### Removing the Add-on

1. Select your desired cluster in the [AWS Elastic Kubernetes Service Console](https://us-west-2.console.aws.amazon.com/eks/home?region=us-west-2#/clusters)
2. Under the "Add-ons" tab, click on the LeakAgent plugin to view the details page.
3. Click "Remove" in the top right-hand corner.
4. Type in the plugin name and press "Remove".

## Installing LeakAgent Add-on using AWS CLI

Run the following command to install the LeakAgent add-on for your Amazon EKS cluster:

```
$ aws eks create-addon --addon-name leaksignal_leakagent --cluster-name $YOUR_CLUSTER_NAME --region $AWS_REGION
```

To monitor the installation status, run the following command:

```
$ aws eks describe-addon --addon-name leaksignal_leakagent --cluster-name $YOUR_CLUSTER_NAME --region $AWS_REGION
```

### Removing the Add-on

To monitor the remove the add-on, run the following command:

```
$ aws eks delete-addon --addon-name leaksignal_leakagent --cluster-name $YOUR_CLUSTER_NAME --region $AWS_REGION

```

## Validating the Install

To validate that LeakAgent has been installed properly, the following command can be run in LeakAgent's namespace (`leakagent` by default) to check the status of all LeakAgent pods:
```
$ kubectl -n leakagent get pods
```

It may take a couple of minutes for all pods to come up but all pods should ultimately display the status of `Running`.

```
$ kubectl -n leakagent get pods
NAME                      READY   STATUS    RESTARTS   AGE
leakagent-748d98b-4bscd   1/1     Running   0          32s
leakagent-748d98b-4zmnh   1/1     Running   0          33s
```

If the pods are not coming online, you can check their logs for more information:

```
$ kubectl -n leakagent logs $POD_NAME
```

If the pods are not able to start, you can check the events on the pod via:

```
# kubectl -n leakagent describe pod $POD_NAME
```

If you have issues getting LeakAgent online, you can send us an email at support@leaksignal.com.

## Getting Metrics

To start getting metrics out of LeakAgent, you'll want to hook up Prometheus, preferably via [kube-prometheus](https://github.com/prometheus-operator/kube-prometheus) or similar. LeakAgent natively integrates with [Prometheus Operator](https://github.com/prometheus-operator/prometheus-operator), and by extension `kube-prometheus`.