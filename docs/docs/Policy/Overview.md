---
sidebar_position: 1
---


## Components

* [Categories](Categories): What kind of sensitive data we are looking for
* [Endpoints](Endpoints/Overview): Where to look for that sensitive data
* [Match Rules](Match%20Rules): Our common format for specifying how to parse, match, and exclude text.
* [Service Identification](Service%20Identification): How we determine what a service is called
* [SBAC](SBAC): Hard blocking rules (i.e. A credit card number is never dispatched to the outside internet)
* [Rules](Rules): Distributed alerts and ratelimits (i.e. Non-admin users cannot see more than 10 distinct credit cards in an hour)
* [Body Collection](Body%20Collection): When to upload entire request/response/stream bodies
* [Header Collection](Header%20Collection): What headers to upload for telemetry
* [Report Style](Report%20Style): Configures what form is match information sent upstream
* [Parsers](Parsers): Interpreting Layer 4 and Layer 7 structure.
* [Environment Collection](Environment): Interpreting Layer 4 and Layer 7 structure.

## Misc Fields

* `path_groups`: A list of [PathGlobs](Endpoints/Path%20Globs) that are used for additional path aggregation.
