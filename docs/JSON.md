# JSON output

`uuidx --output json` emits newline-delimited JSON. Each line
is one independent record, so a batch can be streamed without collecting all
inputs in memory.

Every record contains:

| Field | Meaning |
| --- | --- |
| `schema_version` | Integer JSON contract version, currently `1` |
| `operation` | `generate`, `inspect`, `validate`, or `convert` |
| `index` | Zero-based record index |
| `ok` | Whether this record succeeded |
| `input` | Original input for input-based operations |
| `value` | Normalized or generated text value |
| `bytes` | Lowercase 32-character hexadecimal payload when a UUID/ULID was parsed |
| `kind` | `uuid` or `ulid` when successful |
| `version` | UUID version such as `v4`, `v7`, `nil`, or `max` |
| `format` | Requested output format for generation or conversion |
| `metadata` | Version-specific semantic metadata |
| `fields` | Bit layout entries with bit offsets and hexadecimal values |
| `error` | Structured error with `code` and `message` when `ok` is false |
| `warnings` | Non-fatal policy or legacy-compatibility warnings |

Fields with no value are omitted. A successful UUID inspection contains
`bytes`, `kind`, `version`, `metadata`, and `fields`; a successful ULID
inspection contains `bytes`, `kind`, and ULID metadata. A failed record
contains `input` and `error`.

## Metadata

`metadata.type` identifies the shape:

- `none` for Nil, Max, or values with no additional metadata;
- `random` for v4;
- `name_based` with `algorithm` for v3/v5;
- `time` with Unix timestamp data and optional node information for v1/v6/v7;
- `custom` with final v8 bytes;
- `dce_security` for v2;
- `ulid` with millisecond timestamp and random payload for optional ULID
  inspection.

`inspect --redact-sensitive` omits the `node_id` value from time metadata while
keeping its `node_kind` classification. It does not change the parsed UUID
bytes or the normalized value. Pretty inspection hides bit layout rows by
default; `inspect --layout` displays them. JSON inspection always preserves
the structured `fields` array for automation.

## Example

```json
{"schema_version":1,"operation":"inspect","index":0,"ok":true,"input":"018f2c0b-6c5b-7d2e-8f4a-123456789abc","value":"018f2c0b-6c5b-7d2e-8f4a-123456789abc","bytes":"018f2c0b6c5b7d2e8f4a123456789abc","kind":"uuid","version":"v7","metadata":{"type":"time","unix_seconds":1714430897,"unix_millis":1714430897243,"subsec_nanos":243000000,"clock_sequence":null,"node_id":null,"node_kind":null},"fields":[{"name":"unix_timestamp_ms","offset":0,"width":48,"value":"0x18f2c0b6c5b"}]}
```

Consumers should branch on `schema_version`, `operation`, and `ok`, and ignore
unknown fields so additive fields can be introduced in a future schema.
