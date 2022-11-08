---
sidebar_position: 2
---
A Correlate category composes two other categories (generally Matcher category), and only signals a match if the two match within a certain distance of one another.

## Unformatted phone numbers near "phone"
This example matches 10 digit unformatted phone numbers within 64 bytes of the "phone" string. The `interest` field denotes which of the two groups should be reported as interesting, or if omitted, both groups **and all characters in-between** are considered as matched.

```
categories:
  phone_number_near_label:
    Correlate:
      group1:
        regex_strip: 1
        regexes:
          - "[^0-9][0-9]{10}[^0-9]"
      group2:
        raw:
          - phone
      interest: group1
      max_distance: 64
```

Notably, these groups can also be references to other matching rules:

```
categories:
  phone_number:
    Matchers:
      regex_strip: 1
      regexes:
        - "[^0-9][0-9]{10}[^0-9]"
  phone_number_near_label:
    Correlate:
      group1: phone_number
      group2:
        raw:
          - phone
      interest: group1
      max_distance: 64
```

