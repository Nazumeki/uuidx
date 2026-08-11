mod sections;
mod theme;

use uuidx_core::{Uuid, UuidInspection, UuidOutputFormat};

use self::theme::{error, heading, label, success, version_tag, warning};

#[cfg(feature = "ulid-inspect")]
use self::theme::dim;

pub fn generated(index: u64, value: &str, version: &str, warnings: &[String]) -> String {
    let mut output = format!(
        "{}\n{} {} {}",
        heading("Generated UUID"),
        label(&format!("#{index}")),
        version_tag(version),
        success(value),
    );
    for message in warnings {
        output.push_str(&format!("\n{} {}", warning("warning"), message));
    }
    output
}

pub fn inspection(
    index: u64,
    input: &str,
    inspection: &UuidInspection,
    redact_sensitive: bool,
    show_layout: bool,
) -> String {
    let mut output = format!(
        "{}\n{}",
        heading(&format!("UUID inspection #{index}")),
        sections::summary(input, inspection, redact_sensitive),
    );
    if show_layout {
        output.push_str(&format!("\n{}", sections::layout(&inspection.fields)));
    }
    if let Some(warning_text) = inspection_warning(inspection) {
        output.push_str(&format!("\n{} {}", warning("warning"), warning_text));
    }
    output
}

#[cfg(feature = "ulid-inspect")]
pub fn ulid_inspection(index: u64, input: &str, inspection: &uuidx_core::UlidInspection) -> String {
    format!(
        "{}\n{} {}\n{} {}\n{} {}\n{} {}\n{} {}",
        heading(&format!("ULID inspection #{index}")),
        label("input"),
        input,
        label("normalized"),
        success(&inspection.normalized),
        label("timestamp_ms"),
        inspection.timestamp_ms,
        label("random"),
        dim(&format!("0x{:020x}", inspection.random)),
        warning("status"),
        "inspection-only compatibility",
    )
}

pub fn converted(index: u64, input: &str, value: &str, format: UuidOutputFormat) -> String {
    format!(
        "{}\n{} {}\n{} {}",
        heading(&format!("Converted UUID #{index}")),
        label("input"),
        input,
        label(&format.to_string()),
        success(value),
    )
}

pub fn validated(index: u64, input: &str, uuid: &Uuid) -> String {
    format!(
        "{} {} {}\n{} {}",
        success("[ok]"),
        label(&format!("#{index}")),
        input,
        label("normalized"),
        uuid.hyphenated(),
    )
}

pub fn data_error(index: u64, input: &str, message: &str) -> String {
    format!("{} {}: {}: {}", error("[error]"), index, input, message)
}

pub fn top_level_error(message: &str) -> String {
    format!("{} {}", error("error:"), message)
}

fn inspection_warning(inspection: &UuidInspection) -> Option<&'static str> {
    match inspection.version {
        uuidx_core::InspectableUuidVersion::V1 => {
            Some("UUID v1 exposes timestamp and node metadata; generation is disabled.")
        }
        uuidx_core::InspectableUuidVersion::V2 => {
            Some("UUID v2 DCE Security semantics are outside RFC 9562; generation is disabled.")
        }
        uuidx_core::InspectableUuidVersion::V3 => Some("UUID v3 uses legacy MD5 name hashing."),
        uuidx_core::InspectableUuidVersion::V5 => Some("UUID v5 uses legacy SHA-1 name hashing."),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuidx_core::{inspect_uuid, parse_uuid};

    #[test]
    fn generated_uses_distinct_index_version_and_value_colors() {
        let output = generated(0, "019febdd-f3b3-7d83-9ecb-6ea7c465b028", "v7", &[]);

        assert!(output.contains("\u{1b}[36m#0\u{1b}[0m"));
        assert!(output.contains("\u{1b}[1m\u{1b}[32mv7\u{1b}[0m"));
        assert!(output.contains("\u{1b}[1m\u{1b}[92m019febdd"));
    }

    #[test]
    fn renderers_cover_warnings_and_command_specific_sections() {
        let warning = vec!["application warning".to_owned()];
        let generated_output = generated(3, "value", "v8", &warning);
        assert!(generated_output.contains("#3"));
        assert!(generated_output.contains("application warning"));

        for (input, expected_warning) in [
            (
                "11111111-1111-1111-9111-111111111111",
                "UUID v1 exposes timestamp",
            ),
            (
                "11111111-1111-2111-9111-111111111111",
                "UUID v2 DCE Security",
            ),
            (
                "11111111-1111-3111-9111-111111111111",
                "UUID v3 uses legacy MD5",
            ),
            (
                "11111111-1111-5111-9111-111111111111",
                "UUID v5 uses legacy SHA-1",
            ),
        ] {
            let uuid = parse_uuid(input).unwrap();
            let inspected = inspect_uuid(&uuid);
            let output = inspection(0, input, &inspected, false, false);
            assert!(output.contains(expected_warning), "input: {input}");
        }

        let uuid = parse_uuid("018f2c0b-6c5b-7d2e-8f4a-123456789abc").unwrap();
        let inspected = inspect_uuid(&uuid);
        assert!(inspection(0, &uuid.to_string(), &inspected, false, true).contains("Bit layout"));
        assert!(!inspection(0, &uuid.to_string(), &inspected, false, false).contains("Bit layout"));
        assert!(converted(1, "input", "output", UuidOutputFormat::Urn).contains("Converted UUID"));
        assert!(validated(2, "input", &uuid).contains("[ok]"));
        assert!(data_error(3, "bad", "invalid").contains("[error]"));
        assert!(top_level_error("failure").contains("error:"));
    }

    #[cfg(feature = "ulid-inspect")]
    #[test]
    fn ulid_renderer_marks_read_only_compatibility() {
        let inspected = uuidx_core::inspect_ulid("01ARZ3NDEKTSV4RRFFQ69G5FAV").unwrap();
        let output = ulid_inspection(4, &inspected.normalized, &inspected);
        assert!(output.contains("ULID inspection #4"));
        assert!(output.contains("inspection-only compatibility"));
    }
}
