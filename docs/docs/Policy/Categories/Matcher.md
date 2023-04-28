---
sidebar_position: 1
---
Matchers are data patterns that are searched in a response body. They can be regexes, raw values, exceptions (the `ignore` section), or accelerated native matchers. All individual matching strategies are considered individually, and a match by any matching strategy constitutes a match of the category.


## Writing regexes

Regex implementations (including the one we use, the `fancy_regex` Rust crate) have poor performance when used with _lookaround_ (_lookahead_ or _lookbehind_) regex features. It's advised to instead capture the area around the interesting sensitive data, then strip it out via the `regex_strip` field shown below.

Since the policies are written in YAML, any backslash in a regex must be escaped (`\d` is invalid, where `\\d` matches a digit group as expected). In some cases, it may be more readable to specify character ranges (`[0-9]` as opposed to `\\d`).

## Email regex with an ignored email
This example matches emails, but ignores the specific email `someone@example.com`

```
categories:
  email:
    Matchers:
      regexes:
        - "[a-zA-Z0-9_.+-]{2,}@[a-zA-Z0-9-]{3,}\\.[a-zA-Z0-9-.]{2,}"
      ignore:
        - someone@example.com
```

## Suspicious strings
This example matches some suspicious string literals. These are usually used in JSON keys, or other structured data formats.

```
categories:
  suspicious_strings:
    Matchers:
      raw:
        - credit_card
        - social_security_number
        - password_hash
```

In practice, they might be broken down into multiple rules for more detailed reporting. For instance, the previous example can be rewritten as follows.

```
categories:
  personal_information:
    Matchers:
      raw:
        - credit_card
        - social_security_number
  security_data:
    Matchers:
      raw:
        - password_hash
```

## Case Insensitive Suspicious strings
Similar to above, this matches suspicious string literals. The `case_sensitive` field marks the raw (and only raw) match rules to ignore ASCII case.

```
categories:
  suspicious_strings:
    Matchers:
      raw:
        - credit_card
        - social_security_number
        - password_hash
      case_insensitive: true
```

## Unformatted phone numbers
This example matches 10 digit unformatted phone numbers. It checks for the initial and last character being a non-digit then strips the first and last character from the match data with the `regex_strip` field. This is done to avoid using lookahead/lookbehind, which can be much slower.
```
categories:
  phone_number:
    Matchers:
      regex_strip: 1
      regexes:
        - "[^0-9][0-9]{10}[^0-9]"
```
