use std::{
    str::FromStr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuidx_core::{
    GeneratableUuidVersion, GenerationOptions, Uuid, UuidOutputFormat, parse_hex_array,
};

use crate::{
    cli::GenerateArgs,
    errors::CliError,
    output::{Output, OutputWriter},
};

pub fn run<WOut, WErr>(
    args: &GenerateArgs,
    output: &mut Output<WOut, WErr>,
) -> Result<bool, CliError>
where
    WOut: OutputWriter,
    WErr: OutputWriter,
{
    let version = parse_generation_version(&args.target)?;
    let format: UuidOutputFormat = args.format.into();
    let case = args.case.into();
    let options = build_options(args, version)?;
    let warnings = warnings_for(version);

    for index in 0..args.count {
        let uuid = uuidx_core::generate_uuid(&options)
            .map_err(|error| CliError::Usage(error.to_string()))?;
        output.generated(index, &uuid, &version.to_string(), format, case, &warnings)?;
    }
    Ok(false)
}

fn parse_generation_version(value: &str) -> Result<GeneratableUuidVersion, CliError> {
    match value.to_ascii_lowercase().as_str() {
        "v1" | "1" | "v2" | "2" => Err(CliError::Usage(format!(
            "UUID {value} is inspect-only; generation supports v3, v4, v5, v6, v7, and v8"
        ))),
        other => GeneratableUuidVersion::from_str(other).map_err(CliError::Usage),
    }
}

fn build_options(
    args: &GenerateArgs,
    version: GeneratableUuidVersion,
) -> Result<GenerationOptions, CliError> {
    let mut options = GenerationOptions::new(version);
    options.namespace = args.namespace.as_deref().map(parse_namespace).transpose()?;
    options.name = args.name.as_ref().map(|name| name.as_bytes().to_vec());
    options.node = args.node.as_deref().map(parse_node).transpose()?.flatten();
    options.timestamp = args.timestamp.as_deref().map(parse_timestamp).transpose()?;
    options.custom = args
        .custom
        .as_deref()
        .map(|value| {
            parse_hex_array::<16>(value).map_err(|error| CliError::Usage(error.to_string()))
        })
        .transpose()?;
    Ok(options)
}

fn parse_namespace(value: &str) -> Result<Uuid, CliError> {
    match value.to_ascii_lowercase().as_str() {
        "dns" => Ok(Uuid::NAMESPACE_DNS),
        "url" => Ok(Uuid::NAMESPACE_URL),
        "oid" => Ok(Uuid::NAMESPACE_OID),
        "x500" => Ok(Uuid::NAMESPACE_X500),
        _ => uuidx_core::parse_uuid(value).map_err(|error| CliError::Usage(error.to_string())),
    }
}

fn parse_node(value: &str) -> Result<Option<[u8; 6]>, CliError> {
    if value.eq_ignore_ascii_case("random") {
        Ok(None)
    } else {
        parse_hex_array::<6>(value)
            .map(Some)
            .map_err(|error| CliError::Usage(error.to_string()))
    }
}

fn parse_timestamp(value: &str) -> Result<SystemTime, CliError> {
    if let Ok(milliseconds) = value.parse::<u64>() {
        return UNIX_EPOCH
            .checked_add(Duration::from_millis(milliseconds))
            .ok_or_else(|| CliError::Usage("timestamp is outside the supported range".to_owned()));
    }

    let parsed = OffsetDateTime::parse(value, &Rfc3339)
        .map_err(|error| CliError::Usage(format!("invalid timestamp: {error}")))?;
    let seconds = parsed.unix_timestamp();
    if seconds < 0 {
        return Err(CliError::Usage(
            "timestamp must not be before the Unix epoch".to_owned(),
        ));
    }
    UNIX_EPOCH
        .checked_add(Duration::from_secs(seconds as u64))
        .and_then(|time| time.checked_add(Duration::from_nanos(u64::from(parsed.nanosecond()))))
        .ok_or_else(|| CliError::Usage("timestamp is outside the supported range".to_owned()))
}

fn warnings_for(version: GeneratableUuidVersion) -> Vec<String> {
    match version {
        GeneratableUuidVersion::V3 => vec!["UUID v3 uses legacy MD5 name hashing".to_owned()],
        GeneratableUuidVersion::V5 => vec!["UUID v5 uses legacy SHA-1 name hashing".to_owned()],
        GeneratableUuidVersion::V8 => vec!["UUID v8 uniqueness is application-defined".to_owned()],
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use super::*;

    fn args() -> GenerateArgs {
        GenerateArgs {
            target: "v7".to_owned(),
            count: 1,
            namespace: None,
            name: None,
            node: None,
            timestamp: None,
            custom: None,
            format: crate::cli::UuidFormatArg::Canonical,
            case: crate::cli::TextCaseArg::Lower,
        }
    }

    #[test]
    fn generation_target_parser_accepts_aliases_and_blocks_unsupported_versions() {
        assert_eq!(
            parse_generation_version("3").unwrap(),
            GeneratableUuidVersion::V3
        );
        assert_eq!(
            parse_generation_version("4").unwrap(),
            GeneratableUuidVersion::V4
        );
        assert_eq!(
            parse_generation_version("V8").unwrap(),
            GeneratableUuidVersion::V8
        );
        for target in ["v1", "2"] {
            assert!(matches!(
                parse_generation_version(target),
                Err(CliError::Usage(message)) if message.contains("inspect-only")
            ));
        }
        assert!(matches!(
            parse_generation_version("v9"),
            Err(CliError::Usage(message)) if message.contains("unsupported generation target")
        ));
    }

    #[test]
    fn namespace_parser_supports_named_and_explicit_namespaces() {
        assert_eq!(parse_namespace("dns").unwrap(), Uuid::NAMESPACE_DNS);
        assert_eq!(parse_namespace("URL").unwrap(), Uuid::NAMESPACE_URL);
        assert_eq!(parse_namespace("oid").unwrap(), Uuid::NAMESPACE_OID);
        assert_eq!(parse_namespace("x500").unwrap(), Uuid::NAMESPACE_X500);
        assert!(parse_namespace("not-a-namespace").is_err());
        assert!(parse_namespace("018f2c0b-6c5b-7d2e-8f4a-123456789abc").is_ok());
    }

    #[test]
    fn node_parser_supports_random_and_six_byte_hex_values() {
        assert_eq!(parse_node("random").unwrap(), None);
        assert_eq!(parse_node("RANDOM").unwrap(), None);
        assert_eq!(
            parse_node("010203040506").unwrap(),
            Some([1, 2, 3, 4, 5, 6])
        );
        assert!(parse_node("0102").is_err());
        assert!(parse_node("01020304050z").is_err());
    }

    #[test]
    fn timestamp_parser_supports_millis_and_rfc3339() {
        assert_eq!(
            parse_timestamp("1700000000123")
                .unwrap()
                .duration_since(UNIX_EPOCH)
                .unwrap(),
            Duration::from_millis(1_700_000_000_123)
        );
        assert_eq!(
            parse_timestamp("2024-04-29T22:48:17.243Z")
                .unwrap()
                .duration_since(UNIX_EPOCH)
                .unwrap(),
            Duration::new(1_714_430_897, 243_000_000)
        );
        assert!(matches!(
            parse_timestamp("1969-12-31T23:59:59Z"),
            Err(CliError::Usage(message)) if message.contains("before the Unix epoch")
        ));
        assert!(matches!(
            parse_timestamp("not-a-timestamp"),
            Err(CliError::Usage(message)) if message.contains("invalid timestamp")
        ));
    }

    #[test]
    fn option_builder_decodes_all_generation_inputs() {
        let mut input = args();
        input.namespace = Some("dns".to_owned());
        input.name = Some("uuidx".to_owned());
        input.node = Some("010203040506".to_owned());
        input.timestamp = Some("1700000000123".to_owned());
        input.custom = Some("00112233445566778899aabbccddeeff".to_owned());

        let options = build_options(&input, GeneratableUuidVersion::V8).unwrap();
        assert_eq!(options.namespace, Some(Uuid::NAMESPACE_DNS));
        assert_eq!(options.name, Some(b"uuidx".to_vec()));
        assert_eq!(options.node, Some([1, 2, 3, 4, 5, 6]));
        assert_eq!(
            options
                .timestamp
                .unwrap()
                .duration_since(UNIX_EPOCH)
                .unwrap(),
            Duration::from_millis(1_700_000_000_123)
        );
        assert_eq!(
            options.custom,
            Some([
                0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
                0xee, 0xff
            ])
        );
    }

    #[test]
    fn warnings_are_reserved_for_legacy_or_application_defined_versions() {
        assert_eq!(warnings_for(GeneratableUuidVersion::V3).len(), 1);
        assert!(warnings_for(GeneratableUuidVersion::V4).is_empty());
        assert!(warnings_for(GeneratableUuidVersion::V7).is_empty());
        assert_eq!(warnings_for(GeneratableUuidVersion::V5).len(), 1);
        assert_eq!(warnings_for(GeneratableUuidVersion::V8).len(), 1);
    }

    #[test]
    fn run_generates_values_and_maps_generation_errors_to_usage() {
        let mut input = args();
        input.target = "v4".to_owned();
        input.count = 2;
        let mut output = Output::with_writers(
            &crate::cli::GlobalOptions {
                output: crate::cli::OutputModeArg::Json,
            },
            Vec::new(),
            Vec::new(),
        );
        assert!(!run(&input, &mut output).unwrap());

        let mut input = args();
        input.target = "v5".to_owned();
        input.namespace = Some("dns".to_owned());
        input.name = Some("uuidx".to_owned());
        let mut output = Output::with_writers(
            &crate::cli::GlobalOptions {
                output: crate::cli::OutputModeArg::Json,
            },
            Vec::new(),
            Vec::new(),
        );
        assert!(!run(&input, &mut output).unwrap());

        let mut invalid = args();
        invalid.target = "v1".to_owned();
        let mut output = Output::with_writers(
            &crate::cli::GlobalOptions {
                output: crate::cli::OutputModeArg::Json,
            },
            Vec::new(),
            Vec::new(),
        );
        assert!(matches!(
            run(&invalid, &mut output),
            Err(CliError::Usage(message)) if message.contains("inspect-only")
        ));
    }

    #[test]
    fn run_propagates_generation_and_renderer_errors() {
        let mut invalid = args();
        invalid.target = "v5".to_owned();
        let mut output = Output::with_writers(
            &crate::cli::GlobalOptions {
                output: crate::cli::OutputModeArg::Plain,
            },
            Vec::new(),
            Vec::new(),
        );
        assert!(matches!(
            run(&invalid, &mut output),
            Err(CliError::Usage(message)) if message.contains("requires --namespace and --name")
        ));

        let mut valid = args();
        valid.target = "v4".to_owned();
        let mut output = crate::output::test_output(
            crate::cli::OutputModeArg::Plain,
            crate::output::TestWriter::failing_write(),
            crate::output::TestWriter::working(),
        );
        assert!(matches!(run(&valid, &mut output), Err(CliError::Output(_))));
    }
}
