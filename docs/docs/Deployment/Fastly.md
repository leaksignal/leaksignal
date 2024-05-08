# Backends
- requires a `forward_to` backend with `override_host`
- requires a `command` backend with `override_host`

# Config stores
requires a config store named `leaksignal_config` with the following:
- requires `policy_id` key with corresponding policy_id string. Will fail open if not included.
- requires `policy` key with leaksignal policy serialized as json without newlines. Will fail open if not included.
- optional `block_state` key containing a json serialized `BlockState`. Will use no block state if not included.
- optional `log_level` key containing corresponding log level string. Will use `warn` if not included.

# Secret stores
requires a secret store named `api_key` with the following:
- requires `api_key` with corresponding API key string