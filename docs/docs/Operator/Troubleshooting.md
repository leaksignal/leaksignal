---
sidebar_position: 4
---

## Getting Help

For technical issues you can [file an issue on GitHub](https://github.com/leaksignal/leaksignal-operator/issues). You can also reach out to us at support@leaksignal.com.

## Logs

The LeakSignal Operator exposes logs on a pod in the namespace deployed, i.e. `leaksignal-operator`.

To fetch logs on a Kubernetes deployment in the `leaksignal-operator` namespace, run:

1. `kubectl -n leaksignal-operator get pods`
2. `kubectl -n leaksignal-operator logs leaksignal-operator-0`
