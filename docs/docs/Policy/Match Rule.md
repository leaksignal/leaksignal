---
sidebar_position: 5
---


## Usage

Match Rules are a text-based filter based on one or more strings. They are used for ratelimits and service matching.

## Format

A Match Rule is optionally prefixed with one of the following:
* `regex:`: parsed as regex, automatically anchored to beginning and end of input
* `raw:`: exact match required (default)
* `raw_insensitive:`: exact match required (case insensitive)
* `except:`: can be used to negate previous rules, requires exact match like `raw`
* `except_regex:`: same as `regex`, but also negates previous rules like `except`
* `internal:`: Use a natively implemented matching function.


## Examples

(one rule per line, double line break is a separate set of rules)

```
regex:test[0-7]{3}
except:test000

regex:[a-z]+@[a-z]{2,15}\.[a-z]{2,5}
except:test@example.com
except:example@test.com
except_regex:(?:no-reply|noreply)@.*
```
