---
sidebar_position: 7
---

# Stream Upload Interval

`stream_upload_interval` is a list of rules that determines how frequently match data will be sent when in l4 streaming mode. Once any rule in the list is satisfied, the parser will be flushed and match data will be sent to command.

the options are:
- `frames`: upload matches every `n` frames. requires at least one match to be found.
- `matches`: upload matches when the amount of matches exceeds `n`
- `time` upload matches every `n` ms. requires at least one match to be found.

the default is `matches: 250` and `time: 5000`. no matter what, matches will always get sent when the connection closes.

this example will upload match data when either: 50 matches have been found, 10 seconds has passed and at least one match has been found, or 50 frames have passed and at least one match has been found:

```yaml
stream_upload_interval:
    - matches: 50
    - time: 10000
    - frames: 50
```

# Stream types

similar to `content_types`, `stream_types` can be used to specify what parsers to use with what ports when in streaming mode. it is a mapping of content types to port filters. unless `src`, or `dest` are specified, the filter will check both the source and destination ports. the possible filters are:

- `src`: takes another filter that the source port must match
- `dest`: takes another filter that the destination port must match
- `port`: takes an int that one of the ports must match
- `range`: has `start` and `end` fields that one of the ports must fall between
- `any`: takes an array of filters of which at least one should match
- `all`: takes an array of filters of which all should match
- `not`: takes an array of filters of which none should match
- `always`: always will always match. this exists for cases where you want all traffic to go through a specific parser
- `never`: never will never match. this exists for cases where you want to fully disable one of the default parsers.



the (overwriteable) defaults are:

```yaml
stream_types:
  filebeat:
    dest:
      port: 5044
  text:
    any:
      - port: 80
      - port: 8080
      - port: 8000
```

this means the `filebeat` parser will be used if the destination port is 5044, and the `text` parser will be used if either the source or destination port are 80, 8080, or 8000. the filters are checked in the order that they are defined with the default filters checked last.

if you wanted to disable text parsing you could add the following to your policy to overwrite the default:

```yaml
stream_types:
    text: never
```