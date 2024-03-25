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

## Running as a Service

If you want, you can run leakfile as a service. Heres an example of how to do that for systemd on ubuntu:

### Create Systemd File

Create a new systemd service file for FileLeak at `/etc/systemd/system/FileLeak.service`. Fill it with the following, replacing `User`, `WorkingDirectory`, and `ExecStart` with your own values.

```ini
[Unit]
Description=FileLeak
After=network.target

[Service]
Type=simple
User=yourusername
WorkingDirectory=/path/to/your/app
ExecStart=/path/to/fileleak -u https://your_upstream.com:443 -a your_api_key
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

### Add Service

run

```bash
sudo systemctl daemon-reload
sudo systemctl enable FileLeak.service
sudo systemctl start FileLeak.service
sudo systemctl status FileLeak.service
```

You should see that the service is now running