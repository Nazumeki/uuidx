# JSON output

Use `--output json` (or `-o json`) to emit newline-delimited JSON (JSONL).
Each line is one independent record written to stdout, so input batches can be
processed as a stream. JSON output never contains ANSI escape sequences.

## Record envelope

Every record contains these fields:

| Field | Meaning |
| --- | --- |
| `schema_version` | Integer JSON contract version, currently `1`. |
| `operation` | `generate`, `inspect`, `validate`, or `convert`. |
| `index` | Zero-based index among generated records or nonblank input records. |
| `ok` | `true` for a successful record and `false` for an invalid input record. |

The following fields are present only when the record has a value for them:

| Field | Meaning |
| --- | --- |
| `input` | Trimmed input value for input-based operations. Omitted for generation. |
| `value` | Generated, normalized, or converted text value. |
| `bytes` | Lowercase 32-character hexadecimal representation of a 16-byte UUID or ULID payload. Omitted for NanoIDs and Snowflakes. |
| `kind` | `uuid`, `ulid`, `nanoid`, or `snowflake`. |
| `version` | UUID version: `v1` through `v8`, `nil`, `max`, or `unknown(n)`. Omitted for other families. |
| `format` | Requested UUID output format: `canonical`, `simple`, `urn`, or `braced`. |
| `metadata` | Version- or kind-specific semantic metadata. |
| `fields` | UUID bit-layout entries. |
| `error` | Error object for an invalid input record. |
| `warnings` | Non-fatal warnings. Omitted when the array would be empty. |

Record-level optional fields are omitted, not serialized as `null`. The
optional members inside `metadata.type = "time"` are an exception: they are
always present and use `null` when unavailable.

## Operation records

Successful records use these operation-specific fields:

| Operation | Fields in addition to the envelope |
| --- | --- |
| `generate` | `value`, `bytes`, `kind`, `version`, `format`, and optional `warnings` |
| `inspect` UUID | `input`, `value`, `bytes`, `kind`, `version`, `metadata`, `fields`, and optional `warnings` |
| `inspect` ULID | `input`, `value`, `bytes`, `kind`, and `metadata`; `version` and `fields` are omitted |
| `inspect` NanoID | `input`, `value`, `kind`, and `metadata` |
| `inspect` Snowflake | `input`, `value`, `kind`, and `metadata` |
| `validate` | `input`, `value`, `bytes`, `kind`, and `version` |
| `convert` | `input`, `value`, `bytes`, `kind`, `version`, `format`, and optional `warnings` |

JSON inspection always includes all available metadata and UUID `fields`,
regardless of `--layout`. The option controls only the human-readable pretty
renderer, where it adds raw values, decoded components, and bit ranges.

For UUID records, `value` is canonical text for inspection and validation,
and uses the requested format for generation and conversion. ULID inspection
returns normalized uppercase text. NanoID text is case-sensitive and retained.
Snowflake text is canonical unsigned decimal without leading zeroes.

## Metadata

`metadata` is emitted for successful inspection records. UUID metadata uses a
tagged object whose `type` values and payloads are:

| `type` | Payload |
| --- | --- |
| `none` | No additional metadata. Used for Nil, Max, and unknown UUID versions. |
| `random` | No additional fields. Used for UUID v4. |
| `name_based` | `algorithm`, either `MD5` for v3 or `SHA-1` for v5. |
| `time` | `unix_seconds`, `unix_millis`, `subsec_nanos`, `clock_sequence`, `node_id`, and `node_kind`. Used for v1, v6, and v7. |
| `custom` | `bytes`, the final 16 UUID bytes for v8, in lowercase hexadecimal. |
| `dce_security` | No additional fields. Used for v2. |

For `time` metadata, `clock_sequence` is present for v1 and v6 and is
`null` for v7. `node_id` is a lowercase 12-character hexadecimal value for
v1/v6 when available, and `node_kind` is either
`unicast / may be hardware-derived` or `multicast / locally generated`.
These members are `null` for v7.

`inspect --redact-sensitive` sets `metadata.node_id` to `null` while retaining
`metadata.node_kind`. It does not change `bytes`, `value`, or bit fields.

When the CLI is built with its default `ulid-inspect` feature, ULID inspection
uses this metadata shape:

```json
{
  "type": "ulid",
  "timestamp_ms": 1469922850259,
  "random": "0xd6764c61efb99302bd5b",
  "bytes": "01563e3ab5d3d6764c61efb99302bd5b"
}
```

The ULID `bytes` member duplicates the record-level `bytes` value. ULIDs are
never generated, validated, or converted as UUIDs.

Standard NanoID inspection uses this metadata shape:

```json
{"type":"nanoid","length":21,"alphabet":"A-Za-z0-9_-","entropy_bits":126}
```

Only NanoID's default 21-character URL-safe format is recognized. Since
NanoID supports custom sizes and alphabets, other formats cannot be inferred
reliably from an untagged value.

Original Twitter Snowflake inspection uses this metadata shape:

```json
{
  "type": "snowflake",
  "epoch": "twitter",
  "epoch_ms": 1288834974657,
  "timestamp_ms": 1700000000000,
  "datacenter_id": 17,
  "worker_id": 23,
  "sequence": 3210
}
```

The decoder assumes Twitter's original 41-bit millisecond timestamp, 5-bit
datacenter, 5-bit worker, and 12-bit sequence layout. Custom Snowflake epochs
or bit allocations cannot be inferred automatically. The full identifier
remains in the record-level string `value`.

## Bit fields

Each UUID `fields` entry has this shape:

| Field | Meaning |
| --- | --- |
| `name` | Semantic layout field name. |
| `offset` | Starting bit offset, counted from the most significant bit of the 128-bit UUID. |
| `width` | Number of bits in the field. |
| `value` | Lowercase hexadecimal field value with a `0x` prefix. |

Fields are emitted in this order for each UUID version:

| Version | Field names |
| --- | --- |
| v1 | `time_low`, `time_mid`, `version`, `time_hi`, `variant`, `clock_sequence`, `node` |
| v6 | `timestamp_high`, `version`, `timestamp_low`, `variant`, `clock_sequence`, `node` |
| v7 | `unix_timestamp_ms`, `version`, `rand_a`, `variant`, `rand_b` |
| v8 | `custom_a`, `version`, `custom_b`, `variant`, `custom_c` |
| v2, v3, v4, v5, and unknown versions | `payload_a`, `version`, `payload_b`, `variant`, `payload_c` |
| Nil and Max | `value` |

The offsets and widths in the record are authoritative; the field names above
describe the current layout generated by the core inspector. Non-UUID records
do not contain `fields`.

## Warnings

Warnings are informational strings and are emitted only when non-empty. The
current warnings are:

| Condition | Warning |
| --- | --- |
| Generate, inspect, or convert UUID v3 | `UUID v3 uses legacy MD5 name hashing` |
| Generate, inspect, or convert UUID v5 | `UUID v5 uses legacy SHA-1 name hashing` |
| Generate UUID v8 | `UUID v8 uniqueness is application-defined` |
| Inspect UUID v1 | `UUID v1 exposes timestamp and node metadata` |
| Inspect UUID v2 | `UUID v2 DCE Security semantics are outside RFC 9562` |
| Inspect UUID v3 | `UUID v3 uses legacy MD5 name hashing` |

## Errors and batches

An invalid input produces a record with `ok: false`, `input`, and:

```json
"error": {
  "code": "invalid_uuid",
  "message": "..."
}
```

The current data-error codes are:

| Operation | Code |
| --- | --- |
| `inspect` | `invalid_identifier` |
| `validate` or `convert` | `invalid_uuid` |

`message` is a human-readable parser or inspection error and should not be
used as the machine-readable discriminator.

Positional values, file input, and piped stdin are processed in order. Values
are trimmed, blank values are ignored, and indexes remain contiguous over the
records that are emitted. Without `--fail-fast`, processing continues after a
data error and the command returns exit status `1`. With `--fail-fast`, it
stops after the first invalid record. A successful batch returns `0`.

Usage and missing-input failures return `2`; input, output, and JSON
serialization failures return `3`. These top-level failures are reported on
stderr and do not produce JSON records.

## Examples

UUID v7 inspection:

```json
{"schema_version":1,"operation":"inspect","index":0,"ok":true,"input":"018f2c0b-6c5b-7d2e-8f4a-123456789abc","value":"018f2c0b-6c5b-7d2e-8f4a-123456789abc","bytes":"018f2c0b6c5b7d2e8f4a123456789abc","kind":"uuid","version":"v7","metadata":{"type":"time","unix_seconds":1714430897,"unix_millis":1714430897243,"subsec_nanos":243000000,"clock_sequence":null,"node_id":null,"node_kind":null},"fields":[{"name":"unix_timestamp_ms","offset":0,"width":48,"value":"0x18f2c0b6c5b"},{"name":"version","offset":48,"width":4,"value":"0x7"},{"name":"rand_a","offset":52,"width":12,"value":"0xd2e"},{"name":"variant","offset":64,"width":2,"value":"0x2"},{"name":"rand_b","offset":66,"width":62,"value":"0xf4a123456789abc"}]}
```

A mixed validation batch:

```json
{"schema_version":1,"operation":"validate","index":0,"ok":true,"input":"018f2c0b-6c5b-7d2e-8f4a-123456789abc","value":"018f2c0b-6c5b-7d2e-8f4a-123456789abc","bytes":"018f2c0b6c5b7d2e8f4a123456789abc","kind":"uuid","version":"v7"}
{"schema_version":1,"operation":"validate","index":1,"ok":false,"input":"not-a-uuid","error":{"code":"invalid_uuid","message":"invalid UUID: invalid character: found `n` at 0"}}
```

Consumers should branch on `schema_version`, `operation`, and `ok`, and ignore
unknown fields so additive fields can be introduced in a future schema.
