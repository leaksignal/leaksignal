---
sidebar_position: 12
---

# Environment Collection

`collected_env_vars` is used to filter what environment variables are sent to command. Every version of leaksignal has its own list of default values it will send if this field isn't specified. Here are some example policies:

send all values:
```yaml
collected_env_vars: all
```

send `PATH` and `PWD` variables, alongside the default ones normally sent by the target:
```yaml
collected_env_vars:
    - "PATH"
    - "PWD"
```

send no values, not even the default ones:
```yaml
collected_env_vars: none
```

Some versions of leaksignal insert their own pseudo-environment variables, like the leaksignal target name. These values skip the filtering process entirely and cannot be disabled.