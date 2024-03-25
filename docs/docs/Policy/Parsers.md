---
sidebar_position: 11
---

LeakSignal defines a parser as a structured view into a HTTP request body, HTTP response body, or either direction of a TCP stream.

## Parser Selection

### HTTP

LeakSignal looks at the MIME type in the `content-type` header of the request and response body as the primary hint to which parser to use for the body.

This can be configured via the `content_types` policy field, the defaults are as follows:
```yaml
content_types:
- text/html: text
- text/plain: text
- text/xml: text
- application/soap+xml: text
- application/atom+xml: text
- application/xhtml+xml: text
- application/vnd.mozilla.xul+xml: text
- application/xml: text
- application/vnd.openxmlformats-officedocument.spreadsheetml.sheet: text
- application/vnd.openxmlformats-officedocument.presentationml.presentation: text
- application/grpc: grpc
- application/grpc+proto: grpc
- application/json: json
- application/ld+json: json
```

### TCP

Similar to `content_types`, `stream_types` can be used to specify what parsers to use with what ports when in streaming mode. It is a mapping of content types to port filters. unless `src`, or `dest` are specified, the filter will check both the source and destination ports.

The possible filters are:

- `src`: takes another filter that the source port must match
- `dest`: takes another filter that the destination port must match
- `port`: takes an int that one of the ports must match
- `range`: has `start` and `end` fields that one of the ports must fall between
- `any`: takes an array of filters of which at least one should match
- `all`: takes an array of filters of which all should match
- `not`: takes an array of filters of which none should match
- `always`: always will always match. this exists for cases where you want all traffic to go through a specific parser
- `never`: never will never match. this exists for cases where you want to fully disable one of the default parsers.

The (overwriteable) defaults are:

```yaml
stream_types:
  filebeat:
    dest:
      port: 5044
  fluentd:
    dest:
      port: 24224
  text:
    any:
      - port: 80
      - port: 8080
      - port: 8000
```

This means the `filebeat` parser will be used if the destination port is 5044, `fluentd` will be used if the destination is 24224, and the `text` parser will be used if either the source or destination port are 80, 8080, or 8000. The filters are checked in the order that they are defined with the default filters checked last.

If you wanted to disable text parsing you could add the following to your policy to overwrite the default:

```yaml
stream_types:
    text: never
```

### File System

If using the LeakFile proxy, you can use the `file_types` to set which files to parse and what parsers to use

```yaml
file_types:
  /usr/**/*.log: text
  coding/work/my_log.txt: text
```

#### Telemetry Flushing

With TCP streaming and LeakFile parsing, connections are long lived, and a request/response model is not always applicable. To allow consistent telemetry upload to Command, we define `stream_upload_interval` on the policy.

`stream_upload_interval` is a filter that determines how frequently match data will be sent when in TCP streaming mode. Once the rule is satisfied, the parser will be flushed and telemetry will be sent to Command. No matter what is put here, data will always be sent when the connection is closed.

Options:
- `frames`: upload matches every `n` frames
- `matches`: upload matches when the amount of matches exceeds `n`
- `time` upload matches every `n` ms
- `all`: a list of interval filters of which all should match
- `any`: a list of interval filters of which at least one should match
- `none`: do not perform interval matching and instead wait for the end of the stream to send any data.

This example will upload match data when 50 matches have been found and either 10 seconds has passed or 50 frames have passed:
```yaml
stream_upload_interval:
    all:
      - matches: 50
      - any:
        - time: 10000
        - frames: 50
```

If `stream_upload_interval` was not specified, the default is:
```yaml
stream_upload_interval:
    any:
      - matches: 250
      - time: 5000
```

## Current Parsers

### Text

The fallback parser for unknown formats. Matches on the entire content without attempting to identify structure.

### JSON

Parses one or more JSON documents without buffering.

### GRPC

Parses a GRPC request/response body. Does not require a protobuf, and decodes the protobuf transparently.

### Filebeat

Parses a TCP stream containing Lumberjack V1/V2 protocol, which is commonly used between Filebeat and LogStash and other systems. Internally uses the JSON parser.

### Tls

Marks a TCP stream as containing TLS or not via the `connection_info` field, `ls_tls`.

### Msgpack

Parses a generic Msgpack body or stream.

### Fluentd

Parses a TCP stream containing Fluentd Forward protocol, which is commonly used between Fluentd and other Fluentd instances or other log aggregators. Internally uses the Msgpack parser.

### None

Doesn't parse the body/stream at all. Uses as an explicit disable.