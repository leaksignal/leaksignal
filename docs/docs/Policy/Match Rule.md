---
sidebar_position: 5
---


# Usage

Match Rules are a text-based filter based on one or more strings. They are used for rate limits and service matching.

## Format

A Match Rule is optionally prefixed with one of the following:

* `regex:`: parsed as regex, automatically anchored to beginning and end of input
* `raw:`: exact match required (default)
* `raw_insensitive:`: exact match required (case insensitive)
* `except:`: can be used to negate previous rules, requires exact match like `raw`
* `except_regex:`: same as `regex`, but also negates previous rules like `except`
* `internal:`: Use a natively implemented matching function.
* `and`: a list of rules. overrides the behavior of the group so that a match is only emitted if a rule in the group matches AND every rule in the `and` rule matches
* `correlate`: Allows you to specify a sub-group of rules. Matches from the parent group will only be emitted when a match from the sub-group is found nearby.

## Internal matchers

Internal matchers can be specified via the "internal" match rule. They allow you to use highly optimized dedicated matchers for common match cases or cases too complex for regex to easily handle. The currently implemented internal matchers are:

- `routing_number`: Matches on bank routing numbers
- `credit_card`: Matches on credit/debit card numbers. Supports almost ever major bank, and requires the number to have a valid LUHN checksum
- `int_phone`: Matches on international phone numbers via googles [libphonenumber](https://github.com/google/libphonenumber) library. Requires the country code to be specified beforehand (ie `+1` or `+33`). Does not support IDD codes, does not support full RFC3966 syntax (like extensions).
- `national_phone`: Matches on phone numbers specific to a country. Requires the country ID to be specified (ie `US`, `NL`). Since you specify the country in the policy, the number doesn't require the country code (ie `+1`) to be specified. NOTE: This is a "secondary matcher", meaning it can only be used inside `and` MatchRules. This means you need to specify a custom matcher to match for the phone number format of your desired country, and THEN specify `national_phone` in the `and` matcher to perform a full check.

## Examples

(one rule per line, double line break is a separate set of rules)

```yaml
regex: test[0-7]{3}
except: test000

regex: [a-z]+@[a-z]{2,15}\.[a-z]{2,5}
except: test@example.com
except: example@test.com
except_regex: (?:no-reply|noreply)@.*


regex: "\\b\\d{3}[ .-]\\d{2}[ .-]\\d{4}\\b"
correlate:
  interest: primary
  max_distance: 16
  matches:
    - raw_insensitive: ssn
correlate:
  interest: secondary
  max_distance: 16
  matches:
    - raw_insensitive: social
    - raw_insensitive: security
```

`national_phone` matcher for the US:
```yaml
phone:
  - regex: "(?u)\\b(1[ .-]?)?[2-9]\\d{2}[ .-]?\\d{3}[ .-]?\\d{4}\\b|(\\b1[ .-]?)?\\([2-9]\\d{2}\\)[ .-]?\\d{3}[ .-]?\\d{4}\\b"
  - and:
    - internal: !national_phone US
```