---
sidebar_position: 3
---

## Detecting the First Signs of Serious Exploits

In today's cybersecurity landscape, a critical aspect of safeguarding your infrastructure is the ability to detect the first signs of serious exploits as they occur. LeakSignal excels at identifying these initial indicators by analyzing live traffic and responses to early-stage attacks.

### How Exploits Work

Cybercriminals often begin their attacks by using automated scanning tools to find vulnerable web servers. Once a vulnerability is detected, they execute commands on the compromised server to confirm the exploit. This feedback loop is a crucial stage where the attack can be detected early.

#### Common Scanning Tools

1. **Nmap**: Identifies open ports and services, highlighting potential vulnerabilities.
2. **OpenVAS**: Scans for known vulnerabilities in web servers and applications.
3. **Nikto**: Detects server misconfigurations, outdated software, and insecure settings.
4. **Burp Suite**: Finds vulnerabilities like XSS and SQL injection in web applications.
5. **Metasploit**: Exploits detected vulnerabilities in web servers and applications.
6. **Cobalt Strike**: Used by attackers to gain initial access, move laterally, and deploy ransomware.

### Detecting Initial Exploits with LeakSignal

When an attacker successfully executes a command on a vulnerable server, the response contains unique signals that indicate the exploit. LeakSignal can detect these signals in real-time, providing an early warning system for serious exploits.

#### Example: Detecting Ransomware Feedback Loops

Attackers often run commands like `ls`, `cat`, or `ifconfig` to gather information from the compromised server. LeakSignal can be configured to detect the unique outputs of these commands. For instance, the following policy detects the output of common reconnaissance commands:

```yaml
categories:
  rce_ls:
    - regex: "(?-u:\\b)drwx"
  rce_ifconfig:
    - regex: "(?-u:\\b)ether "
  rce_root:
    - regex: "(?-u:\\b)root(?-u:\\b)"
  rce_privatekey:
    - regex: PRIVATE KEY
```

When any of these rules are triggered, LeakSignal alerts you in the COMMAND dashboard and can send notifications via SMS or email.

#### Example: Detecting Remote Command Execution (RCE)

RCE vulnerabilities allow attackers to execute commands on a server via crafted URLs. Detecting these exploits is challenging but crucial. LeakSignal can identify RCE attempts by analyzing response signatures. Here’s an example of a policy detecting `ls` command output:

```yaml
categories:
  rce_ls_root:
    - regex: "\\broot root\\b"
    - regex: "\\bdrwxr-xr-x\\b"
    - regex: "\\btotal [0-9]+\\b"
  rce_ifconfig:
    - regex: "\\beth0\\b"
    - regex: "\\binet [0-9]+\\.[0-9]+\\.[0-9]+\\.[0-9]+\\b"
    - regex: "\\bnetmask [0-9]+\\.[0-9]+\\.[0-9]+\\.[0-9]+\\b"
  rce_private_key:
    - regex: "-----BEGIN RSA PRIVATE KEY-----"
    - regex: "-----BEGIN PRIVATE KEY-----"
  rce_etc_passwd:
    - regex: "\\broot:x:0:0:root:/root:/bin/bash\\b"
    - regex: "\\bnobody:x:65534:65534:nobody:/nonexistent:/usr/sbin/nologin\\b"
  rce_uname:
    - regex: "\\bLinux [a-zA-Z0-9]+ [0-9]+\\.[0-9]+\\.[0-9]+\\b"
    - regex: "\\bSMP [a-zA-Z]+ [0-9]{4}\\b"

```
This policy detects various signals that are indicative of a successful exploit:

- **rce_ls_root**: Looks for common directory listing outputs such as "root root," permissions strings, and total counts.
- **rce_ifconfig**: Matches network interface configuration outputs like "eth0," IP addresses, and netmask values.
- **rce_private_key**: Detects private key headers in responses.
- **rce_etc_passwd**: Identifies lines from the `/etc/passwd` file indicating user information.
- **rce_uname**: Matches system information output from the `uname -a` command, including kernel version and build info.

LeakSignal can also detect specific system outputs, such as those from Microsoft Exchange vulnerabilities like ProxyLogin and ProxyNotShell:

```yaml
categories:
  rce_exchange:
    - regex: "\\bnt authority\/system\\b"
```

LeakSignal offers robust detection capabilities across various protocols, including API, HTTP, gRPC, and WebSocket traffic. By monitoring for the unique signals generated by initial exploit commands, LeakSignal provides a critical layer of defense, enabling you to respond swiftly and effectively to emerging threats.

This approach ensures that your security team is alerted at the first signs of an exploit, allowing for immediate investigation and mitigation to protect your infrastructure from further damage.