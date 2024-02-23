---
sidebar_position: 2
---

Categories are data patterns that are searched in a response body. They can be regexes, raw strings, excluding rules, correlate rules, or accelerated native matchers.

The following matching strategies are considered individually, and a match by any matching strategy constitutes a match of the category:

- raw (String),
- rawInsensitive (String),
- regex (Regex),
- internal (native matcher),

The possible values for `internal` are:

- routing_number
- credit_card

The following have special properties that will override the behavior of other rules in the category:

- and (list of other match rules),
- except_regex (Regex),
- except (String),
- exceptInsensitive (String),
- correlate (correlate rule),

## Writing regexes

Since the policies are written in YAML, any backslash in a regex must be escaped (`\d` is invalid, where `\\d` matches a digit group as expected). In some cases, it may be more readable to specify character ranges (`[0-9]` as opposed to `\\d`).

## Email regex with an ignored email

This example matches emails, but ignores the specific email `someone@example.com`

```yaml
categories:
  email:
    - regex: "[a-zA-Z0-9_.+-]{2,}@[a-zA-Z0-9-]{3,}\\.[a-zA-Z0-9-.]{2,}"
    - except: someone@example.com
```

## Suspicious strings

This example matches some suspicious string literals. These are usually used in JSON keys, or other structured data formats.

```yaml
categories:
  suspicious_strings:
    - raw: credit_card
    - raw: social_security_number
    - raw: password_hash
```

In practice, they might be broken down into multiple rules for more detailed reporting. For instance, the previous example can be rewritten as follows.

```yaml
categories:
  personal_information:
    - raw: credit_card
    - raw: social_security_number
  security_data:
    - raw: password_hash
```

## Case Insensitive Suspicious strings

Similar to above, this matches suspicious string literals while ignoring ASCII case.

```yaml
categories:
  suspicious_strings:
    - raw_insensitive: credit_card
    - raw_insensitive: social_security_number
    - raw_insensitive: password_hash
```

## Unformatted phone numbers

This example matches 10 digit unformatted phone numbers

```yaml
categories:
  phone_number:
    - regex: "[^0-9][0-9]{10}[^0-9]"
```

## Correlate rules

A Correlate rule contains a secondary rule group, and only signals a match if the parent group and secondary group match within a certain distance of one another. This effects the behavior of the entire category.

### Unformatted phone numbers near "phone"

This example matches 10 digit unformatted phone numbers within 64 bytes of the "phone" string. The `interest` field denotes which of the two groups should be reported as interesting: `primary` (default), `secondary`, or `all` for all characters between both matches.

```yaml
categories:
  phone_number_near_label:
    - regex: "[^0-9][0-9]{10}[^0-9]"
    - correlate:
        interest: all
        max_distance: 16
        matches:
          - raw: phone
```

Notably, these groups can also be references to other matching rules:

```yaml
categories:
  phone_number:
    - regex: "[^0-9][0-9]{10}[^0-9]"
  phone_number_near_label:
    - raw: "number"
    - correlate:
        interest: secondary
        max_distance: 16
        match_group: phone_number
```

### Multiple Correlates

You can specify multiple correlates within the same category, which will be evaluated separately from each-other. Nesting correlate rules inside other correlate rules is not supported and will throw an error during policy deserialization.

```yaml
categories:
  ssn:
    - regex: "\\b\\d{3}[ .-]\\d{2}[ .-]\\d{4}\\b"
    - correlate:
        interest: primary
        max_distance: 16
        matches:
          - raw_insensitive: ssn
    - correlate:
        interest: secondary
        max_distance: 16
        matches:
          - raw_insensitive: social
          - raw_insensitive: security
```


## Category Tags

There is an alternative form of parsing for categories that allows you to set a tag for the corresponding matchers. if used, the tag will be added to the metadata of any produced matches.

The below example will set `"tag": "routing"` in the metadata field of any matches produced by `routing` or `routing_2`. As contrast, the `phone_number` category still uses the normal form of category parsing.

```yaml
categories:
  phone_number:
    - regex: "[^0-9][0-9]{10}[^0-9]"
  routing:
    matchers: !internal routing_number
    tag: "routing"
  routing_2:
    matchers:
      - regex: "\\b\\d{9}\\b"
      - regex: "\\b\\d{5}\\b"
      - and: !internal routing_number
    tag: "routing"
```