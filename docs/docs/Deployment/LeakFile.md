---
sidebar_position: 5
---

LeakFile is a version of leaksignal that can be run against files on the local system, such as log files.

## Running

LeakFile takes the following arguments

```
Usage: leakfile [OPTIONS]

Options:
  -u, --upstream <UPSTREAM>  Address of command server
  -a, --api-key <API_KEY>    API key for command server
  -p, --policy <POLICY>      optional policy. if this is set then the client will be run as local and NOT talk to command
  -h, --help                 Print help
```

## Configuration

LeakFile receives polices from command the same way any other version of leaksignal does. It will look at the [file_types](../Policy/Parsers.md#file-system) field in the policy to determine what parsers to use with what files. When LeakFile first opens a file, it will start parsing from the end of the file, so only *new* content will be scanned.
