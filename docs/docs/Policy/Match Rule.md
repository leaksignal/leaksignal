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
* `and`: group multiple rules into one rule
* `correlate`: Allows you to specify a sub-group of rules. Matches from the parent group will only be emitted when a match from the sub-group is found nearby.

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
