---
sidebar_position: 4
---

# Token Extraction

- `location`: the [Token Extraction Site](#Token%20Extraction%20Site). Defines the location of the extracted token.
- `header`: the name of the header to be extracted
- `extractor`: a list of [TokenExtractor](#Token%20Extractor)s to be executed in order


## Token Extraction Site
- `request`
- `request_cookie`
- `response`


## Token Extractor

An extraction instruction that takes in a string, performs an extraction on it, and outputs a string. Meant to be chained together.

- `regex`: performs a given regex on a string and outputs the First capture group of first match, or entire first match if no capture group
- `jwt_decode`: extracts the payload of a jwt token
- `json_path`: extracts a value from a json body using the given [JsonPath](https://goessner.net/articles/JsonPath/)
- `hash`: hashes the currently extracted value with SHA256. intended to be used at the end of a set of extraction instructions
- `metadata`: works differently than the rest of the extractors. this will "fork" the extractor at the current extraction step and parse any amount of tokens you want from it, then insert them into the `token_metadata` field of the `MatchData` output. This is great if you have any additional metadata you want to extract from a token without actually setting that value as the token.


# Examples:
generic example that extracts the first 4 letters of the "name" field in a jwt token
``` yaml
token_extractor:
    location: request
    header: jwt
    extractor:
    - jwt_decode
    - json_path: $.name
    - regex: \b[a-zA-Z]{4}\b
```

a similar example, except this one extracts multiple bits of metadata during the main tokens extraction

```yaml
token_extractor:
  location: request
  header: jwt
  extractor:
    - jwt_decode
    # extra metadata to extract after the jwt has been decoded but before the rest of the token is extracted
    - metadata:
        # extracts some digits
        "a": !regex \d+
        # this step wont actually produce anything since the token were operating over will fail multiple jwt_decodes
        "b":
          - jwt_decode
          - jwt_decode
          - jwt_decode
        # extracts the contents of the "sub" key then hashes it
        "c":
          - json_path: $.sub
          - hash
    - json_path: $.name
    - regex: \b[a-zA-Z]{4}\b
```

an example of what the output of the above policy might look like in the `MatchData` output from leaksignal:
```json
"token": "John",
"token_metadata": {
    "c": "C775E7B757EDE630CD0AA1113BD102661AB38829CA52A6422AB782862F268646",
    "a": "1234567890"
}
```