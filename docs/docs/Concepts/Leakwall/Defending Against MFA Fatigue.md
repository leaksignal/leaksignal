---
sidebar_position: 1
---

A new threat is emerging in the industry known as the MFA fatigue attack, which exploits social engineering tactics. This attack involves cybercriminals bombarding the targeted users with multiple mobile push notifications requesting approval for login. By overwhelming the targets with repeated requests, attackers hope to deceive them into clicking "accept" at least once, allowing them to access the corporate accounts.

Due to the continuous onslaught of malicious MFA push requests, many users often give in to the pressure and inadvertently approve the login attempts or accept them to put an end to the endless alerts, enabling the attackers to gain access to the accounts.

Lapsus$ and Yanluowang threat actors have already demonstrated the effectiveness of this social engineering attack by using it to infiltrate high-profile organizations such as Microsoft, Cisco, and Uber.

## How It Works

When user credentials are breached and MFA is achieved, attackers are given an authentication token. This auth token is then used to access sensitive data and systems that only the employee who received the MFA fatigue attack would be able to view.

This is where LeakSignal monitoring starts. Post-MFA, after the MFA Fatigue attack has taken place, LeakSignal Sentry correlates the auth token to the amount of sensitive data that is accessed.

## Token-based Sensitive Data Exfiltration Alerting in LeakSignal

The following policy provides observability and alerting on the amount of credit card data that is accessed by a single token over a 5 minute period.

```yaml
categories:
  bank_credit_card_data:
    - regex: "(?-u:\\b)(?:4[0-9]{12}(?:[0-9]{3})?|[25][1-7][0-9]{14}|6(?:011|5[0-9][0-9])[0-9]{12}|3[47][0-9]{13}|3(?:0[0-5]|[68][0-9])[0-9]{11}|(?:2131|1800|35\\d{3})\\d{11})(?-u:\\b)"
    - except: 0000-0000-0000-0000
endpoints:
  - matches: "**"
    config:
      bank_credit_card_data:
        report_style: partial_sha256
        report_bits: 32
    token_extractor:
      location: request
      header: Authorization
      regex: "Token ([^\\.]+\\.[^\\.]+)\\.[^\\.]+"
      hash: true
rules:
  - grouping: global
    by: token
    filter:
      all:
        - response_outbound
        - response_matches:
            credit_card: 1
    name: "Per Token CC"
    severity: immediate
    action: alert
    timespan_secs: 300
    limit: 5
```

This is a simple example to demonstrate the power of sensitive data monitoring after MFA occurrs. LeakSignal supports many other rule sets around sensitive data access and exfiltration.
