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

similar to `content_types`, `stream_types` can be used to specify what parsers to use with what ports. the (overwriteable) defaults are:

```yaml
stream_types:
    5044: filebeat
    80: text
    8080: text
    8000: text
```

