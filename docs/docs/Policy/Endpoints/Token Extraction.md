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


## Examples:
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

# Body Extraction

Similar to token extraction, leaksignal also supports extracting metadata from specific fields in a request/response body. The way this works is simpler than Token Extraction, you just supply a mapping of keys and json paths and any value that matches that path during parsing will have its value placed in the resulting proto.

## Example 1:

```yaml
endpoints:
  - request_extractors:
      "ssn":
        - "test.my_ssn3[*].my_ssn4"
        - "test.my_ssn2"
      "never": "aaa.aaa.aaa.aaa.aaa.hello"
    response_extractors:
      "ssn2": "test.my_ssn3[*].my_ssn4"
```

An example of what the output of the above policy might look like in the `MatchData` output from leaksignal. Here `blob_idx` refers to the index of the json body for cases where a single payload will contain multiple separate json objects:
```json
"matches": {
  "response": {
    "body_metadata": [
      {
        "key": "ssn",
        "value": "123-45-4892",
        "blob_idx": 0
      },
      {
        "key": "ssn",
        "value": "123-45-4895",
        "blob_idx": 0
      }
    ]
  }
}
```

## Example 2:

While body extraction was originally designed to pull whole values from json fields, an alternative syntax exists that allows you to use a regex to extract sub-values from the selected fields like so:

```yaml
endpoints:
  response_extractors:
    "ssn":
      path: "test.my_ssn3[*].my_ssn4"
      regex: "\\d{4}"
    "never":
      - "hello.aaa.aaa.aaa.aaa.aaa"
      - "goodbye.aaa.aaa.aaa.aaa.aaa"
```

As you can see, the `ssn` extractor uses the alternative syntax to extract the last 4 digits of a social security number from the selected field.

## Body Extraction Notes

Body extractors like this exist to help give context to matches. Because of that, by default, values will only be extracted from blobs that contain matches. If you want to override this behavior, you can use the following alternative syntax to set the `include_all` flag and retrieve metadata from every blob regardless of matches.
```yaml
endpoints:
  request_extractors:
    extractors:
      "ssn":
        - "test.my_ssn3[*].my_ssn4"
        - "test.my_ssn2"
      "never": "aaa.aaa.aaa.aaa.aaa.hello"
    include_all: true
  response_extractors:
    "ssn2": "test.my_ssn3[*].my_ssn4"
```

As you can see, several alternate syntaxes exist for body extraction depending on the desired behavior, and any combination of the formats can be used. The following are some examples of possible ways they can be combined:

```yaml
response_extractors:
  "test": "hello.aaa"
```
```yaml
response_extractors:
  "test":
    - "hello.aaa"
    - "goodbye.aaa"
```
```yaml
response_extractors:
  "test":
    path: "hello.aaa"
    regex: "\\d{4}"
```
```yaml
request_extractors:
  extractors:
    "test":
        - path: "hello.aaa"
          regex: "\\d{4}"
        - path: "goodbye.aaa"
          regex: ".{4}"
  include_all: true
```