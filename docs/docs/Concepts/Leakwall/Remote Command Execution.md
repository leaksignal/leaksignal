---
sidebar_position: 1
---

Remote Command Execution (RCE) is one of the most dangerous types of vulnerabilities. It allows attackers to execute commands on a server by simply requesting a URL with a crafted payload. To execute an RCE and eventually takeover a machine, one simply sends a request to a URL similar to the following:
```
http:/yourwebsite.com/vulnerableAPI/attack?search=<query>
```
Using the latest Text4Shell vulnerability as an example, an attacker would send the following request to an exposed server:
```
http:/yourwebsite.com/vulnerableAPI/attack?search=${script:javascript:java.lang.Runtime.getRuntime().exec('ls -e /bin/sh')}
```
These types of requests trigger undetected exploits in underlying production code. In the cases of log4shell in 2021 and the Equifax breach of 2017, RCEs were found in the underlying components that were widely used across many public facing software systems worldwide - and these types of exploits are [only getting worse](https://www.cisa.gov/known-exploited-vulnerabilities-catalog).

## RCE Detection

RCE detection can be difficult due to many factors. Some detection solutions require agent installation or instrumentation of an existing codebase. These solutions become cumbersome, hard to implement consistently across large organizations, and are an overall maintenance nightmare.

Additionally, third party (supply chain) scanning is a good preventitive measure, but not effective in preventing zero days and flawed API logic susceptible to an RCE attack.
There are no silver bullets to defend against every type of RCE, but response analysis is one of the most efficacious approaches.

When attackers are probing to check for RCE vulnerabilities, they're looking for indicators of a successful exploit. Many probing and scanning tools send a malicious request to check for system output. This could be in the form of a `ls`, `cat`, or `ifconfig` linux command. The commands will generate responses with unique signals that can be detected with low false positive rates.
Here's an example of the `ls` command used in a scanning attack:

![directory listing output](../../../static/img/ls-output.png)

To detect this type of infrastructure probing, a LeakSignal category can be created in the policy to detect the signature of `ls` output.
```
categories:
  rce_ls_root:
    Matchers:
      regexes:
        - "\\broot root\\b"
```
:::note
See [Policy](/Policy) documentation for more information on how policies work.
:::

Additionally, response signatures can be customized to match the output of specific systems like Microsoft Exchange. The following example shows how exploitation of ProxyLogin and ProxyNotShell could be detected in the outgoing HTTP response.

![exchange proxylogin](https://github.com/leaksignal/leaksignal/raw/master/assets/proxylogin-output.png)

```
categories:
  rce_ls_root:
    Matchers:
      regexes:
        - "\\bnt authority\/system\\b"
```
LeakSignal can detect response signatures across all mesh API, HTTP, gRPC and WebSocket traffic.

