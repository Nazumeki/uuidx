mod sections;
mod theme;

use uuidx_core::{IdentifierInspection, UuidInspection, UuidOutputFormat};

use self::theme::{error, heading, label, success, version_tag, warning};

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
        sections::summary(input, inspection),
    );
    if show_layout {
        output.push_str(&format!(
            "\n{}",
            sections::details(inspection, redact_sensitive)
        ));
        output.push_str(&format!("\n{}", sections::layout(&inspection.fields)));
    }
    if let Some(warning_text) = inspection_warning(inspection) {
        output.push_str(&format!("\n{} {}", warning("warning"), warning_text));
    }
    output
}

#[cfg(feature = "ulid-inspect")]
pub fn ulid_inspection(
    index: u64,
    input: &str,
    inspection: &uuidx_core::UlidInspection,
    show_layout: bool,
) -> String {
    let mut output = format!(
        "{}\n{}",
        heading(&format!("ULID inspection #{index}")),
        sections::identifier_summary(
            input,
            &inspection.normalized,
            &[(
                "timestamp",
                sections::format_unix_millis(inspection.timestamp_ms),
            )],
        ),
    );
    if show_layout {
        output.push_str(&format!(
            "\n{}",
            sections::identifier_details(&[
                ("bytes", dim(&hex::encode(inspection.bytes))),
                ("timestamp_ms", inspection.timestamp_ms.to_string()),
                ("random", dim(&format!("0x{:020x}", inspection.random))),
            ]),
        ));
        output.push_str(&format!("\n{}", sections::layout(&inspection.fields)));
    }
    output
}

pub fn nanoid_inspection(
    index: u64,
    input: &str,
    inspection: &uuidx_core::NanoidInspection,
    show_layout: bool,
) -> String {
    let mut output = format!(
        "{}\n{}",
        heading(&format!("NanoID inspection #{index}")),
        sections::identifier_summary(
            input,
            &inspection.normalized,
            &[("length", inspection.length.to_string())],
        ),
    );
    if show_layout {
        output.push_str(&format!(
            "\n{}",
            sections::identifier_details(&[
                ("alphabet", dim(inspection.alphabet)),
                ("entropy_bits", inspection.entropy_bits.to_string()),
            ]),
        ));
    }
    output
}

pub fn snowflake_inspection(
    index: u64,
    input: &str,
    inspection: &uuidx_core::SnowflakeInspection,
    show_layout: bool,
) -> String {
    let mut output = format!(
        "{}\n{}",
        heading(&format!("Snowflake inspection #{index}")),
        sections::identifier_summary(
            input,
            &inspection.normalized,
            &[(
                "timestamp",
                sections::format_unix_millis(inspection.timestamp_ms),
            )],
        ),
    );
    if show_layout {
        output.push_str(&format!(
            "\n{}",
            sections::identifier_details(&[
                ("value_hex", dim(&format!("0x{:016x}", inspection.value))),
                ("timestamp_ms", inspection.timestamp_ms.to_string()),
                ("epoch_ms", inspection.epoch_ms.to_string()),
                ("datacenter_id", inspection.datacenter_id.to_string()),
                ("worker_id", inspection.worker_id.to_string()),
                ("sequence", inspection.sequence.to_string()),
            ]),
        ));
        output.push_str(&format!("\n{}", sections::layout(&inspection.fields)));
    }
    output
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

pub fn validated(index: u64, input: &str, inspection: &IdentifierInspection) -> String {
    format!(
        "{} {} {}\n{} {}",
        success("[ok]"),
        label(&format!("#{index}")),
        input,
        label("normalized"),
        inspection.normalized(),
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
            Some("UUID v1 exposes timestamp and node metadata.")
        }
        uuidx_core::InspectableUuidVersion::V2 => {
            Some("UUID v2 DCE Security semantics are outside RFC 9562.")
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
        assert!(validated(2, "input", &IdentifierInspection::Uuid(inspected)).contains("[ok]"));
        let nanoid = uuidx_core::inspect_nanoid("V1StGXR8_Z5jdHi6B-myT").unwrap();
        assert!(validated(3, "input", &IdentifierInspection::Nanoid(nanoid)).contains("[ok]"));
        assert!(data_error(4, "bad", "invalid").contains("[error]"));
        assert!(top_level_error("failure").contains("error:"));
    }

    #[cfg(feature = "ulid-inspect")]
    #[test]
    fn ulid_renderer_separates_summary_from_layout() {
        let inspected = uuidx_core::inspect_ulid("01ARZ3NDEKTSV4RRFFQ69G5FAV").unwrap();
        let summary = ulid_inspection(4, &inspected.normalized, &inspected, false);
        assert!(summary.contains("ULID inspection #4"));
        assert!(summary.contains("timestamp"));
        assert!(!summary.contains("timestamp_ms"));
        assert!(!summary.contains("Bit layout"));
        assert!(!summary.contains("status"));

        let layout = ulid_inspection(4, &inspected.normalized, &inspected, true);
        assert!(layout.contains("timestamp_ms"));
        assert!(layout.contains("random"));
        assert!(layout.contains("Bit layout"));
    }

    #[test]
    fn nanoid_and_snowflake_renderers_separate_summary_from_layout() {
        let nanoid = uuidx_core::inspect_nanoid("V1StGXR8_Z5jdHi6B-myT").unwrap();
        let summary = nanoid_inspection(5, &nanoid.normalized, &nanoid, false);
        assert!(summary.contains("NanoID inspection #5"));
        assert!(summary.contains("length"));
        assert!(!summary.contains("entropy_bits"));
        assert!(!summary.contains("profile"));
        let layout = nanoid_inspection(5, &nanoid.normalized, &nanoid, true);
        assert!(layout.contains("alphabet"));
        assert!(layout.contains("entropy_bits"));

        let snowflake = uuidx_core::inspect_snowflake("1724552287438348288").unwrap();
        let summary = snowflake_inspection(6, &snowflake.normalized, &snowflake, false);
        assert!(summary.contains("Snowflake inspection #6"));
        assert!(summary.contains("timestamp"));
        assert!(!summary.contains("datacenter_id"));
        assert!(!summary.contains("profile"));
        let layout = snowflake_inspection(6, &snowflake.normalized, &snowflake, true);
        assert!(layout.contains("datacenter_id"));
        assert!(layout.contains("worker_id"));
        assert!(layout.contains("sequence"));
        assert!(layout.contains("Bit layout"));
    }
}
