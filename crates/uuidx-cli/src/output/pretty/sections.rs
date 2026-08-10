use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuidx_core::{BitField, TimestampInfo, UuidInspection, UuidMetadata, UuidOutputFormat};

use super::theme::{dim, heading, label, success};

const LABEL_WIDTH: usize = 17;

pub fn summary(input: &str, inspection: &UuidInspection, redact_sensitive: bool) -> String {
    let mut lines = vec![
        field("input", input.to_owned()),
        field("normalized", success(&inspection.normalized)),
        field("version", inspection.version.to_string()),
        field("format", input_format(input).to_string()),
        field("variant", inspection.variant.to_string()),
        field("bytes", dim(&hex::encode(inspection.bytes))),
        field("nil", inspection.is_nil.to_string()),
        field("max", inspection.is_max.to_string()),
    ];

    match &inspection.metadata {
        UuidMetadata::Time {
            timestamp,
            clock_sequence,
            node_id,
            node_kind,
        } => {
            lines.push(field("timestamp", format_timestamp(timestamp)));
            lines.push(field("unix_millis", timestamp.unix_millis.to_string()));
            if let Some(sequence) = clock_sequence {
                lines.push(field("clock_sequence", sequence.to_string()));
            }
            if let Some(node) = node_id {
                let value = if redact_sensitive {
                    "[redacted]".to_owned()
                } else {
                    hex::encode(node)
                };
                lines.push(field("node_id", value));
            }
            if let Some(kind) = node_kind {
                lines.push(field("node_kind", kind.to_string()));
            }
        }
        UuidMetadata::NameBased { algorithm } => {
            lines.push(field("algorithm", algorithm.to_string()));
        }
        UuidMetadata::Custom { bytes } => {
            lines.push(field("custom_bytes", hex::encode(bytes)));
        }
        UuidMetadata::DceSecurity => {
            lines.push(field("semantics", "DCE Security".to_owned()));
        }
        UuidMetadata::Random | UuidMetadata::None => {}
    }

    lines.join("\n")
}

pub fn layout(fields: &[BitField]) -> String {
    let mut lines = vec![heading("Bit layout")];
    let field_header = format!("{}{}", label("field"), " ".repeat(24 - 5));
    let bits_header = format!("{}{}", " ".repeat(8 - 4), label("bits"));
    lines.push(format!(
        "  {} {} {}",
        field_header,
        bits_header,
        label("value")
    ));
    for field in fields {
        lines.push(format!(
            "  {:<24} {:>8} {}",
            field.name,
            format!("{}..{}", field.offset, field.offset + field.width - 1),
            dim(&format!("0x{:x}", field.value)),
        ));
    }
    lines.join("\n")
}

fn field(name: &str, value: String) -> String {
    format!(
        "  {}{} {}",
        label(name),
        " ".repeat(LABEL_WIDTH.saturating_sub(name.len())),
        value
    )
}

fn format_timestamp(timestamp: &TimestampInfo) -> String {
    let date_time = OffsetDateTime::from_unix_timestamp(timestamp.unix_seconds as i64)
        .ok()
        .and_then(|date_time| date_time.replace_nanosecond(timestamp.subsec_nanos).ok());
    date_time
        .and_then(|date_time| date_time.format(&Rfc3339).ok())
        .unwrap_or_else(|| format!("unix:{}", timestamp.unix_seconds))
}

fn input_format(input: &str) -> UuidOutputFormat {
    let trimmed = input.trim();
    let lowercase = trimmed.to_ascii_lowercase();

    if lowercase.starts_with("urn:uuid:") {
        UuidOutputFormat::Urn
    } else if trimmed.starts_with('{') && trimmed.ends_with('}') {
        UuidOutputFormat::Braced
    } else if trimmed.contains('-') {
        UuidOutputFormat::Canonical
    } else {
        UuidOutputFormat::Simple
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuidx_core::{inspect_uuid, parse_uuid};

    fn inspection(input: &str) -> UuidInspection {
        let uuid = parse_uuid(input).expect("test UUID should parse");
        inspect_uuid(&uuid)
    }

    #[test]
    fn summary_renders_time_name_custom_dce_and_plain_metadata() {
        let v7 = inspection("018f2c0b-6c5b-7d2e-8f4a-123456789abc");
        let v7_summary = summary("018f2c0b-6c5b-7d2e-8f4a-123456789abc", &v7, false);
        assert!(v7_summary.contains("timestamp"));
        assert!(v7_summary.contains("unix_millis"));
        assert!(!v7_summary.contains("node_id"));

        let v6 = inspection("11111111-1111-6111-9111-111111111111");
        let unredacted = summary("11111111-1111-6111-9111-111111111111", &v6, false);
        assert!(unredacted.contains("clock_sequence"));
        assert!(unredacted.contains("node_id"));
        assert!(!unredacted.contains("[redacted]"));
        let redacted = summary("11111111-1111-6111-9111-111111111111", &v6, true);
        assert!(redacted.contains("[redacted]"));

        assert!(
            summary(
                "11111111-1111-5111-9111-111111111111",
                &inspection("11111111-1111-5111-9111-111111111111"),
                false,
            )
            .contains("algorithm")
        );
        assert!(
            summary(
                "11111111-1111-8111-9111-111111111111",
                &inspection("11111111-1111-8111-9111-111111111111"),
                false,
            )
            .contains("custom_bytes")
        );
        assert!(
            summary(
                "11111111-1111-2111-9111-111111111111",
                &inspection("11111111-1111-2111-9111-111111111111"),
                false,
            )
            .contains("DCE Security")
        );
        assert!(
            !summary(
                "11111111-1111-4111-9111-111111111111",
                &inspection("11111111-1111-4111-9111-111111111111"),
                false,
            )
            .contains("algorithm")
        );
    }

    #[test]
    fn summary_detects_all_supported_input_formats() {
        let cases = [
            ("  URN:UUID:018f2c0b-6c5b-7d2e-8f4a-123456789abc ", "urn"),
            ("{018f2c0b-6c5b-7d2e-8f4a-123456789abc}", "braced"),
            ("018f2c0b-6c5b-7d2e-8f4a-123456789abc", "canonical"),
            ("018f2c0b6c5b7d2e8f4a123456789abc", "simple"),
        ];
        for (input, expected) in cases {
            let expected = match expected {
                "urn" => UuidOutputFormat::Urn,
                "braced" => UuidOutputFormat::Braced,
                "canonical" => UuidOutputFormat::Canonical,
                "simple" => UuidOutputFormat::Simple,
                _ => unreachable!(),
            };
            assert_eq!(input_format(input), expected);
        }
    }

    #[test]
    fn layout_and_timestamp_formatting_cover_empty_and_fallback_cases() {
        let fields = vec![BitField {
            name: "payload".to_owned(),
            offset: 0,
            width: 8,
            value: 0xab,
        }];
        assert!(layout(&fields).contains("0..7"));
        assert!(layout(&[]).contains("Bit layout"));

        let valid = TimestampInfo {
            unix_seconds: 1_700_000_000,
            unix_millis: 1_700_000_000_123,
            subsec_nanos: 123_000_000,
        };
        assert!(format_timestamp(&valid).contains("2023-"));
        assert_eq!(
            format_timestamp(&TimestampInfo {
                unix_seconds: i64::MAX as u64,
                unix_millis: 0,
                subsec_nanos: 0,
            }),
            format!("unix:{}", i64::MAX as u64)
        );
        assert_eq!(
            format_timestamp(&TimestampInfo {
                unix_seconds: 0,
                unix_millis: 0,
                subsec_nanos: 1_000_000_000,
            }),
            "unix:0"
        );
    }
}
