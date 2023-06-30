---
sidebar_position: 3
---

## Getting Help

For technical issues you can [file an issue on GitHub](https://github.com/leaksignal/leaksignal/issues). You can also reach out to us at support@leaksignal.com.

## Logs

The LeakSignal Operator exposes logs on a pod in the namespace deployed, i.e. `kube-system`.

To fetch logs on a Kubernetes deployment in the `kube-system` namespace, run:

1. `kubectl -n kube-system get pods`
2. `kubectl -n kube-system logs leaksignal-operator-0`
