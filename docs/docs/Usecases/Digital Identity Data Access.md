---
sidebar_position: 2
---

## Guarding Against Digital Identity Exploits

A growing threat in the industry involves cybercriminals exploiting compromised digital identities to access sensitive data. Attackers bombard users with multiple push notifications requesting login approval, hoping to deceive them into granting access.

When users' credentials are breached and MFA is achieved, attackers gain an authentication token. This token allows them to access sensitive data and systems that the compromised employee would typically view.

## How It Works

After obtaining an authentication token through social engineering or other means, attackers use it to access sensitive information. LeakSignal steps in at this stage, correlating the auth token with the amount of sensitive data accessed post-authentication.

### Token-Based Sensitive Data Monitoring in LeakSignal

LeakSignal provides real-time observability and alerting on sensitive data access. For example, the following policy monitors the amount of credit card data accessed by a single token over a 5-minute period:

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

This example illustrates how LeakSignal monitors sensitive data access post-authentication. By tracking and analyzing data flows associated with digital identities, LeakSignal provides robust protection against unauthorized data access and exfiltration.