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
