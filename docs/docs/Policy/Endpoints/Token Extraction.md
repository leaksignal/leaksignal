---
sidebar_position: 4
---

# Token Extraction

- `location`: the [Token Extraction Site](#Token%20Extraction%20Site). Defines the location of the extracted token.
- `header`: the name of the header to be extracted
- `extractor`: a list of [TokenExtractor](#Token%20Extractor)s to be executed in order
- `hash`: a boolean for if the output should be hashed or not


## Token Extraction Site
- `request`
- `request_cookie`
- `response`
  

## Token Extractor

An extraction instruction that takes in a string, performs an extraction on it, and outputs a string. Meant to be chained together.

- `regex`: performs a given regex on a string and outputs the first match
- `jwt_decode`: extracts the payload of a jwt token
- `json_path`: extracts a value from a json body using the given [JsonPath](https://goessner.net/articles/JsonPath/)


# Example:
``` yaml
token_extractor:
    location: request
    header: jwt
    extractor:
    - jwt_decode
    - json_path: $.name
    - regex: \b[a-zA-Z]{4}\b
```