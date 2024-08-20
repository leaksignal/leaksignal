# Fastly

## Configuration
Leakfastly requires some setup to get running properly. Most of this is automatically handled through command, but the following needs to be set up manually:

## Initial Service Setup

### Backends
- Requires a `forward_to` backend with `override_host`. this is the address that traffic will be forwarded to.
- Requires a `command` backend with `override_host`. this is the address that match data will be sent to.

### Optional Local Flags
The proxy has some optional configurable flags that can be set on a per-service basis. If you want to set them, you can create a dictionary on your service called `leaksignal_flags` and input any of the following keys:
- `log_level` key containing corresponding log level string. Will use `warn` if not set.

## Command configuration

Inside command, you can set your account API key, as well as the ID of the leakfastly services you want command to interface with. From there, command will set up your account with the necessary vk/secret stores, as well as update the specified services to use them. This will allow your leakfastly services to receive the latest policy changes and block list items.
