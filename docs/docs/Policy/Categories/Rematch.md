---
sidebar_position: 3
---
A Rematch category composes two other categories (generally Matcher category). It matches the first category, then the second category in sequence. This can be used to improve performance as an alternative to a complex regex.

It is currently disabled, but support will be re-enabled in the future.

## Credit card validity check
This example matches 4 dash-separated groups of 4 digits (a credit card). It then validates against a regex that checks for a valid credit card.

```
categories:
  credit_card_rematched:
    Rematch:
      target:
        regex_strip: 1
        regexes:
          - "[^0-9]\\d{4}[\\s.-]\\d{4}[\\s.-]\\d{4}[\\s.-]\\d{4}[^0-9]"
          - "[^0-9]\\d{16}[^0-9]"
      rematcher:
        regexes:
          - "(?:4[0-9]{12}(?:[0-9]{3})?|[25][1-7][0-9]{14}|6(?:011|5[0-9][0-9])[0-9]{12}|3[47][0-9]{13}|3(?:0[0-5]|[68][0-9])[0-9]{11}|(?:2131|1800|35\\d{3})\\d{11})"
```

Here too, the `target` and `rematcher` can be references to other matching rules:

```
categories:
  credit_card_loose:
      Matchers:
        regex_strip: 1
        regexes:
          - "[^0-9]\\d{4}[\\s.-]\\d{4}[\\s.-]\\d{4}[\\s.-]\\d{4}[^0-9]"
          - "[^0-9]\\d{16}[^0-9]"
  credit_card_strict:
      Matchers:
        regexes:
          - "(?:4[0-9]{12}(?:[0-9]{3})?|[25][1-7][0-9]{14}|6(?:011|5[0-9][0-9])[0-9]{12}|3[47][0-9]{13}|3(?:0[0-5]|[68][0-9])[0-9]{11}|(?:2131|1800|35\\d{3})\\d{11})"
  credit_card:
    Rematch:
      target: credit_card_loose
      rematcher: credit_card_strict
```
