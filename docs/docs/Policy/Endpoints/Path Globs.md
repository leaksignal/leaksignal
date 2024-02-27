---
sidebar_position: 2
---

# Path Globs

Path globs are similar to a host-prefixed HTTP path. They are used to meaningfully differentiate request URLs in policies.

## Components

A path glob is made up of forward-slash-separated components, with no trailing or leading slash. The first component is protocol specific: in HTTP/gRPC it's the `:authority` or `Host` header. The rest of the components are the HTTP path, not including the query string.

Each component can be one of the following:

* `*`: Matches any single component.
* `**`: Matches an arbitrary number of components (0 or more). This is the only path glob component that can match a variable number of components.
* `#<regex>`: Matches the given regex against the component. Forward slashes are not allowed.
* `*suffix`: Matches if the component ends with the suffix
* `prefix*`: Matches if the component starts with the prefix
* `*within*`: Matches if the component contains the text
* `text`: Matches if the component equals the text.

Additionally, components can use `!` beforehand to specify that the path should be preserved in leaksignal's output.
i.e. given the path `hello/world/test`; the PathGlob `hello/*/test.html` would display as `hello/*/test.html`
but `hello/!*/test.html` would display as `hello/world/test.html`

## Ordering

PathGlobs are sorted for evaluation on specificity. This means that a PathGlob like `**` can be superseded by a PathGlob like `*/test.html`

## Examples

```yaml
# matches any path
**

# matches the path /foo on any hostname
*/foo

# matches any path on the 'example.com' hostname
example.com/**

# matches a parameter component
# i.e. example.com/product/123 OR example.com/product/ABC
example.com/product/*

# matches a regex limited component
# i.e. example.com/product/123 BUT NOT example.com/product/ABC
example.com/product/#[0-9]+

# matches any path ending in '.php'
# the last component must end with '.php', but the rest of the components are ignored
**/*.php
```
