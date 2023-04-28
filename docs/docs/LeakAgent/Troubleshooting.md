---
sidebar_position: 3
---

## Getting Help

For technical issues you can [file an issue on GitHub](https://github.com/leaksignal/leaksignal/issues). You can also reach out to us at support@leaksignal.com.

## Logs

LeakAgent exposes logs on the `leakagent` pods.

To fetch logs on a Kubernetes deployment in the `leakagent` namespace, run:

1. `kubectl -n leakagent get pods`
2. `kubectl -n leakagent logs <pod name>`

## Checking Telemetry Directly

If you are not seeing LeakAgent metrics in Prometheus, you can check if LeakAgent is properly emitting metrics:

1. Find full name of `leakagent` prometheus service: `kubectl -n leakagent get svc`
2. Port forward prometheus`kubectl -n leakagent port-forward svc/leakagent-prom 9176`
3. In a browser, navigate to `http://localhost:9176/metrics` to confirm presence of Prometheus metrics.

If you are seeing metrics here and not in Prometheus, then there is likely a configuration issue with Prometheus. If you are not seeing metrics here, then it is likely that there is some issue with the policy, or Envoy sidecars running LeakSignal.

## Recovery

If there is a critical issue with LeakAgent, you can always remove & redeploy it, or simply delete all deployed pods to reset their state. Please file us a bug report on GitHub if you encounter any issues!