use std::{
    str::FromStr,
    time::{Duration, UNIX_EPOCH},
};

use serde::Deserialize;
use uuidx_core::{
    GeneratableUuidVersion, GenerationOptions, UuidOutputFormat, UuidTextCase,
    format_uuid_with_case, generate_uuid, parse_hex_array, parse_namespace as core_parse_namespace,
    parse_uuid,
};

use crate::error::ApiError;

const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct GenerationInput {
    namespace: Option<String>,
    name: Option<String>,
    node: Option<String>,
    timestamp_ms: Option<f64>,
    custom: Option<String>,
    format: Option<String>,
    case: Option<String>,
}

pub(crate) fn generate(version: &str, input: GenerationInput) -> Result<String, ApiError> {
    let version = GeneratableUuidVersion::from_str(version)
        .map_err(|message| ApiError::new("unsupported_version", message))?;
    let format = parse_format(input.format.as_deref())?;
    let case = parse_case(input.case.as_deref())?;
    let options = build_options(version, input)?;
    let uuid = generate_uuid(&options)
        .map_err(|error| ApiError::new("generation_failed", error.to_string()))?;
    Ok(format_uuid_with_case(&uuid, format, case))
}

pub(crate) fn format(input: &str, format: &str, case: Option<&str>) -> Result<String, ApiError> {
    let uuid = parse_uuid(input)?;
    let format = parse_format(Some(format))?;
    let case = parse_case(case)?;
    Ok(format_uuid_with_case(&uuid, format, case))
}

fn build_options(
    version: GeneratableUuidVersion,
    input: GenerationInput,
) -> Result<GenerationOptions, ApiError> {
    let mut options = GenerationOptions::new(version);
    options.namespace = input
        .namespace
        .as_deref()
        .map(parse_namespace)
        .transpose()?;
    options.name = input.name.map(String::into_bytes);
    options.node = input.node.as_deref().map(parse_node).transpose()?.flatten();
    options.timestamp = input.timestamp_ms.map(parse_timestamp).transpose()?;
    options.custom = input
        .custom
        .as_deref()
        .map(|value| {
            parse_hex_array::<16>(value)
                .map_err(|error| ApiError::new("invalid_custom", error.to_string()))
        })
        .transpose()?;
    Ok(options)
}

fn parse_namespace(value: &str) -> Result<uuidx_core::Uuid, ApiError> {
    core_parse_namespace(value)
        .map_err(|error| ApiError::new("invalid_namespace", error.to_string()))
}

fn parse_node(value: &str) -> Result<Option<[u8; 6]>, ApiError> {
    if value.eq_ignore_ascii_case("random") {
        Ok(None)
    } else {
        parse_hex_array::<6>(value)
            .map(Some)
            .map_err(|error| ApiError::new("invalid_node", error.to_string()))
    }
}

fn parse_timestamp(milliseconds: f64) -> Result<std::time::SystemTime, ApiError> {
    if !milliseconds.is_finite()
        || milliseconds < 0.0
        || milliseconds.fract() != 0.0
        || milliseconds > MAX_SAFE_INTEGER
    {
        return Err(ApiError::new(
            "invalid_timestamp",
            "timestampMs must be a non-negative safe integer",
        ));
    }

    UNIX_EPOCH
        .checked_add(Duration::from_millis(milliseconds as u64))
        .ok_or_else(|| {
            ApiError::new(
                "invalid_timestamp",
                "timestampMs is outside the supported range",
            )
        })
}

fn parse_format(value: Option<&str>) -> Result<UuidOutputFormat, ApiError> {
    match value {
        Some(value) => UuidOutputFormat::from_str(value)
            .map_err(|message| ApiError::new("unsupported_format", message)),
        None => Ok(UuidOutputFormat::Canonical),
    }
}

fn parse_case(value: Option<&str>) -> Result<UuidTextCase, ApiError> {
    match value {
        Some(value) => UuidTextCase::from_str(value)
            .map_err(|message| ApiError::new("unsupported_case", message)),
        None => Ok(UuidTextCase::Lower),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(json: &str) -> GenerationInput {
        serde_json::from_str(json).expect("valid generation input")
    }

    #[test]
    fn maps_every_supported_generation_contract() {
        let cases = [
            ("v3", r#"{"namespace":"dns","name":"example.org"}"#),
            ("v4", "{}"),
            ("v5", r#"{"namespace":"url","name":"https://example.org"}"#),
            (
                "v6",
                r#"{"node":"010203040506","timestampMs":1700000000123}"#,
            ),
            ("v7", r#"{"timestampMs":1700000000123}"#),
            ("v8", r#"{"custom":"00112233445566778899aabbccddeeff"}"#),
        ];

        for (version, options) in cases {
            let generated = generate(version, input(options)).expect("generation should succeed");
            assert_eq!(
                parse_uuid(&generated).unwrap().get_version_num(),
                version[1..].parse::<usize>().unwrap()
            );
        }
    }

    #[test]
    fn name_based_generation_is_deterministic_and_supports_named_namespaces() {
        let first = generate("v3", input(r#"{"namespace":"dns","name":"example.org"}"#)).unwrap();
        let second = generate("v3", input(r#"{"namespace":"dns","name":"example.org"}"#)).unwrap();
        assert_eq!(first, second);
        assert_eq!(first, "04738bdf-b25a-3829-a801-b21a1d25095b");
    }

    #[test]
    fn generation_formats_results_and_rejects_unused_options() {
        let simple = generate("v7", input(r#"{"format":"simple"}"#)).unwrap();
        assert_eq!(simple.len(), 32);

        let upper_urn = generate(
            "v5",
            input(r#"{"namespace":"dns","name":"example.org","format":"urn","case":"upper"}"#),
        )
        .unwrap();
        assert_eq!(upper_urn, "urn:uuid:AAD03681-8B63-5304-89E0-8CA8F49461B5");

        let error = generate("v4", input(r#"{"name":"unused"}"#)).unwrap_err();
        assert_eq!(error.code(), "generation_failed");
    }

    #[test]
    fn generation_reports_each_input_error_at_the_adapter_boundary() {
        let cases = [
            ("v2", "{}", "unsupported_version"),
            ("v4", r#"{"format":"compact"}"#, "unsupported_format"),
            ("v4", r#"{"case":"title"}"#, "unsupported_case"),
            (
                "v3",
                r#"{"namespace":"not-a-namespace","name":"value"}"#,
                "invalid_namespace",
            ),
            ("v6", r#"{"node":"not-a-node"}"#, "invalid_node"),
            ("v7", r#"{"timestampMs":-1}"#, "invalid_timestamp"),
            ("v8", r#"{"custom":"abcd"}"#, "invalid_custom"),
        ];

        for (version, options, expected_code) in cases {
            assert_eq!(
                generate(version, input(options)).unwrap_err().code(),
                expected_code
            );
        }
    }

    #[test]
    fn namespaces_accept_all_aliases_case_insensitively_and_explicit_uuids() {
        use uuidx_core::Uuid;
        for (namespace, expected) in [
            ("DNS", Uuid::NAMESPACE_DNS),
            ("url", Uuid::NAMESPACE_URL),
            ("OID", Uuid::NAMESPACE_OID),
            ("x500", Uuid::NAMESPACE_X500),
            ("6ba7b810-9dad-11d1-80b4-00c04fd430c8", Uuid::NAMESPACE_DNS),
        ] {
            assert_eq!(parse_namespace(namespace).unwrap(), expected);
        }
    }

    #[test]
    fn nodes_accept_random_and_fixed_identifiers() {
        assert_eq!(parse_node("random").unwrap(), None);
        assert_eq!(parse_node("RANDOM").unwrap(), None);
        assert_eq!(
            parse_node("010203040506").unwrap(),
            Some([1, 2, 3, 4, 5, 6])
        );
        assert_eq!(parse_node("0102").unwrap_err().code(), "invalid_node");
    }

    #[test]
    fn validates_javascript_timestamp_constraints() {
        for invalid in [f64::NAN, f64::INFINITY, -1.0, 1.5, MAX_SAFE_INTEGER + 1.0] {
            let error = parse_timestamp(invalid).unwrap_err();
            assert_eq!(error.code(), "invalid_timestamp");
        }
        assert_eq!(
            parse_timestamp(1_700_000_000_123.0)
                .unwrap()
                .duration_since(UNIX_EPOCH)
                .unwrap(),
            Duration::from_millis(1_700_000_000_123)
        );

        #[cfg(windows)]
        assert_eq!(
            parse_timestamp(MAX_SAFE_INTEGER).unwrap_err().code(),
            "invalid_timestamp"
        );

        #[cfg(not(windows))]
        assert_eq!(
            parse_timestamp(MAX_SAFE_INTEGER).unwrap(),
            UNIX_EPOCH
                .checked_add(Duration::from_millis(MAX_SAFE_INTEGER as u64))
                .unwrap()
        );
    }

    #[test]
    fn formatting_covers_all_public_formats() {
        const UUID: &str = "018f2c0b-6c5b-7d2e-8f4a-123456789abc";
        assert_eq!(format(UUID, "canonical", None).unwrap(), UUID);
        assert_eq!(
            format(UUID, "simple", None).unwrap(),
            "018f2c0b6c5b7d2e8f4a123456789abc"
        );
        assert_eq!(
            format(UUID, "urn", Some("lower")).unwrap(),
            format!("urn:uuid:{UUID}")
        );
        assert_eq!(
            format(UUID, "urn", Some("upper")).unwrap(),
            "urn:uuid:018F2C0B-6C5B-7D2E-8F4A-123456789ABC"
        );
        assert_eq!(format(UUID, "braced", None).unwrap(), format!("{{{UUID}}}"));
        assert_eq!(
            format(UUID, "braced", Some("upper")).unwrap(),
            "{018F2C0B-6C5B-7D2E-8F4A-123456789ABC}"
        );
    }

    #[test]
    fn formatting_rejects_invalid_uuid_and_format_values() {
        assert_eq!(
            format("not-a-uuid", "canonical", None).unwrap_err().code(),
            "invalid_uuid"
        );
        assert_eq!(
            format("018f2c0b-6c5b-7d2e-8f4a-123456789abc", "compact", None)
                .unwrap_err()
                .code(),
            "unsupported_format"
        );
        assert_eq!(
            format(
                "018f2c0b-6c5b-7d2e-8f4a-123456789abc",
                "canonical",
                Some("mixed")
            )
            .unwrap_err()
            .code(),
            "unsupported_case"
        );
    }

    #[test]
    fn options_reject_unknown_fields() {
        assert!(serde_json::from_str::<GenerationInput>(r#"{"typo":true}"#).is_err());
    }
}
