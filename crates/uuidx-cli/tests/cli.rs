use std::{
    fs,
    io::Write,
    process::{Command, Output, Stdio},
};

use serde_json::Value;
use uuidx_core::{UuidOutputFormat, parse_uuid};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_uuidx"))
        .args(args)
        .output()
        .expect("uuidx should start")
}

fn run_with_stdin(args: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_uuidx"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("uuidx should start");
    child
        .stdin
        .take()
        .expect("stdin should be available")
        .write_all(input.as_bytes())
        .expect("stdin should accept test input");
    child.wait_with_output().expect("uuidx should finish")
}

#[test]
fn generate_v4_is_plain_pipeline_safe() {
    let output = run(&["generate", "v4", "--output", "plain"]);
    assert!(output.status.success());
    let value = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(!value.contains('\u{1b}'));
    let uuid = parse_uuid(value.trim()).expect("generated UUID should parse");
    assert_eq!(uuid.get_version_num(), 4);
}

#[test]
fn generate_defaults_to_v7_and_command_aliases_work() {
    for args in [
        vec!["generate", "--output", "plain"],
        vec!["g", "-o", "plain"],
    ] {
        let output = run(&args);
        assert!(output.status.success(), "command failed: {args:?}");
        let uuid = parse_uuid(String::from_utf8(output.stdout).unwrap().trim())
            .expect("generated value should parse");
        assert_eq!(uuid.get_version_num(), 7);
    }
}

#[test]
fn application_version_uses_lowercase_short_flag() {
    let lower = run(&["-v"]);
    assert!(lower.status.success());
    assert!(
        String::from_utf8(lower.stdout)
            .unwrap()
            .contains(concat!("uuidx ", env!("CARGO_PKG_VERSION")))
    );

    let upper = run(&["-V"]);
    assert_eq!(upper.status.code(), Some(2));

    let subcommand = run(&["generate", "-v"]);
    assert_eq!(subcommand.status.code(), Some(2));
}

#[test]
fn no_argument_prompt_has_no_product_intro() {
    let output = run(&[]);
    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    let prompt = format!("{stdout}{stderr}");
    assert!(prompt.contains("Usage: uuidx"));
    assert!(!prompt.contains("A modern CLI tool for multi-version UUID generation"));
}

#[test]
fn generated_json_contains_bytes_and_format() {
    let output = run(&["generate", "v4", "--output", "json", "--format", "simple"]);
    assert!(output.status.success());
    let record: Value =
        serde_json::from_slice(&output.stdout).expect("generated JSON should parse");
    assert_eq!(record["operation"], "generate");
    assert_eq!(record["format"], "simple");
    assert_eq!(record["bytes"].as_str().map(str::len), Some(32));
    assert_eq!(record["value"].as_str().map(str::len), Some(32));
}

#[test]
fn every_supported_generation_target_is_available() {
    let cases = [
        (
            vec![
                "generate",
                "v3",
                "--namespace",
                "dns",
                "--name",
                "example.org",
                "--output",
                "plain",
            ],
            3,
        ),
        (
            vec![
                "generate",
                "v6",
                "--timestamp",
                "1700000000123",
                "--node",
                "020000000001",
                "--output",
                "plain",
            ],
            6,
        ),
        (
            vec![
                "generate",
                "v7",
                "--timestamp",
                "1700000000123",
                "--output",
                "plain",
            ],
            7,
        ),
        (
            vec![
                "generate",
                "v8",
                "--custom",
                "00112233445566778899aabbccddeeff",
                "--output",
                "plain",
            ],
            8,
        ),
    ];

    for (args, expected_version) in cases {
        let output = run(&args);
        assert!(output.status.success(), "command failed: {args:?}");
        let value = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
        let uuid = parse_uuid(value.trim()).expect("generated value should parse");
        assert_eq!(uuid.get_version_num(), expected_version);
    }
}

#[test]
fn generation_rejects_legacy_versions() {
    for version in ["v1", "v2", "nil", "max"] {
        let output = run(&["generate", version, "--output", "plain"]);
        assert_eq!(output.status.code(), Some(2));
        let error = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
        if version.starts_with('v') && version != "v4" {
            assert!(error.contains("inspect-only"));
        } else {
            assert!(error.contains("unsupported generation target"));
        }
    }
}

#[test]
fn invalid_generation_and_conversion_options_are_usage_errors() {
    let cases = [
        vec!["generate", "v4", "--count", "0"],
        vec!["generate", "v4", "--format", "invalid"],
        vec!["generate", "v4", "--node", "0102"],
        vec!["generate", "v7", "--timestamp", "not-a-timestamp"],
        vec!["generate", "v8", "--custom", "00"],
        vec!["inspect", "018f2c0b-6c5b-7d2e-8f4a-123456789abc", "--json"],
        vec![
            "inspect",
            "018f2c0b-6c5b-7d2e-8f4a-123456789abc",
            "--color",
            "always",
        ],
        vec![
            "convert",
            "018f2c0b-6c5b-7d2e-8f4a-123456789abc",
            "--to",
            "invalid",
        ],
    ];

    for args in cases {
        let output = run(&args);
        assert_eq!(output.status.code(), Some(2), "command: {args:?}");
        assert!(!output.stderr.is_empty(), "command: {args:?}");
    }
}

#[test]
fn v5_generation_is_deterministic() {
    let args = [
        "generate",
        "v5",
        "--namespace",
        "dns",
        "--name",
        "example.com",
        "--output",
        "plain",
    ];
    let first = run(&args);
    let second = run(&args);
    assert!(first.status.success());
    assert_eq!(first.stdout, second.stdout);
    let uuid = parse_uuid(
        std::str::from_utf8(&first.stdout)
            .expect("stdout should be UTF-8")
            .trim(),
    )
    .expect("generated UUID should parse");
    assert_eq!(uuid.get_version_num(), 5);
}

#[test]
fn v3_generation_is_deterministic_and_warns_about_md5() {
    let args = [
        "generate",
        "v3",
        "--namespace",
        "dns",
        "--name",
        "example.org",
        "--output",
        "json",
    ];
    let output = run(&args);
    assert!(output.status.success());
    let record: Value = serde_json::from_slice(&output.stdout).expect("JSON should parse");
    assert_eq!(record["version"], "v3");
    assert_eq!(record["value"], "04738bdf-b25a-3829-a801-b21a1d25095b");
    assert_eq!(
        record["warnings"][0],
        "UUID v3 uses legacy MD5 name hashing"
    );
}

#[test]
fn convert_emits_requested_format() {
    let output = run(&[
        "convert",
        "018f2c0b-6c5b-7d2e-8f4a-123456789abc",
        "--to",
        "simple",
        "--output",
        "plain",
    ]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout)
            .expect("stdout should be UTF-8")
            .trim(),
        "018f2c0b6c5b7d2e8f4a123456789abc"
    );
    assert_eq!(
        uuidx_core::format_uuid(
            &parse_uuid("018f2c0b-6c5b-7d2e-8f4a-123456789abc").unwrap(),
            UuidOutputFormat::Simple
        ),
        "018f2c0b6c5b7d2e8f4a123456789abc"
    );
}

#[test]
fn convert_defaults_to_canonical_format() {
    let output = run(&[
        "convert",
        "018f2c0b6c5b7d2e8f4a123456789abc",
        "--output",
        "plain",
    ]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout)
            .expect("stdout should be UTF-8")
            .trim(),
        "018f2c0b-6c5b-7d2e-8f4a-123456789abc"
    );
}

#[test]
fn inspect_json_is_one_record_per_line_and_has_no_ansi() {
    let output = run(&[
        "inspect",
        "018f2c0b-6c5b-7d2e-8f4a-123456789abc",
        "--output",
        "json",
    ]);
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(!text.contains('\u{1b}'));
    let record: Value = serde_json::from_str(text.trim()).expect("JSON output should parse");
    assert_eq!(record["ok"], true);
    assert_eq!(record["kind"], "uuid");
    assert_eq!(record["version"], "v7");
    assert_eq!(record["bytes"], "018f2c0b6c5b7d2e8f4a123456789abc");
}

#[test]
fn inspect_json_reports_each_uuid_metadata_family() {
    let cases = [
        ("11111111-1111-4111-9111-111111111111", "random"),
        ("11111111-1111-5111-9111-111111111111", "name_based"),
        ("11111111-1111-8111-9111-111111111111", "custom"),
        ("11111111-1111-2111-9111-111111111111", "dce_security"),
    ];
    for (input, metadata_type) in cases {
        let output = run(&["inspect", input, "--output", "json"]);
        assert!(output.status.success(), "input: {input}");
        let record: Value = serde_json::from_slice(&output.stdout).expect("JSON should parse");
        assert_eq!(record["metadata"]["type"], metadata_type);
    }

    let legacy = run(&[
        "inspect",
        "11111111-1111-3111-9111-111111111111",
        "--output",
        "json",
    ]);
    assert!(legacy.status.success());
    let record: Value = serde_json::from_slice(&legacy.stdout).expect("JSON should parse");
    assert_eq!(record["metadata"]["type"], "name_based");
    assert_eq!(record["metadata"]["algorithm"], "MD5");
    assert_eq!(
        record["warnings"][0],
        "UUID v3 uses legacy MD5 name hashing"
    );
}

#[test]
fn pretty_output_is_compact_and_machine_output_is_ansi_free() {
    let pretty = run(&[
        "inspect",
        "018f2c0b-6c5b-7d2e-8f4a-123456789abc",
        "--output",
        "pretty",
    ]);
    assert!(pretty.status.success());
    let pretty_text = String::from_utf8(pretty.stdout).expect("stdout should be UTF-8");
    assert!(!pretty_text.contains('\u{1b}'));
    assert!(!pretty_text.contains('┌'));
    assert!(!pretty_text.contains('│'));
    assert!(pretty_text.contains("timestamp"));
    assert!(!pretty_text.contains("unix_millis"));
    assert!(!pretty_text.contains("bytes"));
    assert!(!pretty_text.contains("Bit layout"));

    let detailed = run(&[
        "inspect",
        "018f2c0b-6c5b-7d2e-8f4a-123456789abc",
        "--output",
        "pretty",
        "--layout",
    ]);
    assert!(detailed.status.success());
    let detailed_text =
        String::from_utf8(detailed.stdout).expect("detailed output should be UTF-8");
    assert!(detailed_text.contains("Details"));
    assert!(detailed_text.contains("bytes"));
    assert!(detailed_text.contains("unix_millis"));
    assert!(detailed_text.contains("Bit layout"));

    let json = run(&[
        "inspect",
        "018f2c0b-6c5b-7d2e-8f4a-123456789abc",
        "--output",
        "json",
    ]);
    assert!(
        !String::from_utf8(json.stdout)
            .expect("stdout should be UTF-8")
            .contains('\u{1b}')
    );
}

#[test]
fn validate_returns_data_error_status_and_json_error_records() {
    let output = run(&[
        "validate",
        "018f2c0b-6c5b-7d2e-8f4a-123456789abc",
        "not-a-uuid",
        "--output",
        "json",
    ]);
    assert_eq!(output.status.code(), Some(1));
    let text = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let records: Vec<Value> = text
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["ok"], true);
    assert_eq!(records[1]["ok"], false);
    assert_eq!(records[1]["error"]["code"], "invalid_uuid");
}

#[test]
fn inspect_and_validate_fail_fast_after_the_first_bad_record() {
    let output = run(&[
        "validate",
        "not-a-uuid",
        "018f2c0b-6c5b-7d2e-8f4a-123456789abc",
        "--output",
        "json",
        "--fail-fast",
    ]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(String::from_utf8(output.stdout).unwrap().lines().count(), 1);

    let output = run(&["inspect", "not-an-identifier", "--output", "json"]);
    assert_eq!(output.status.code(), Some(1));
    let record: Value = serde_json::from_slice(&output.stdout).expect("error JSON should parse");
    assert_eq!(record["error"]["code"], "invalid_identifier");
}

#[test]
fn pretty_inspection_contains_semantic_sections() {
    let output = run(&[
        "inspect",
        "018f2c0b-6c5b-7d2e-8f4a-123456789abc",
        "--output",
        "pretty",
    ]);
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(text.contains("UUID inspection"));
    assert!(text.contains("version"));
    assert!(text.lines().any(|line| {
        let mut fields = line.split_whitespace();
        fields.next() == Some("format") && fields.next() == Some("canonical")
    }));
    assert!(text.contains("timestamp"));
    assert!(!text.contains("unix_millis"));
    assert!(!text.contains("Bit layout"));
}

#[test]
fn pretty_inspection_reports_input_format() {
    for (input, expected) in [
        ("018f2c0b-6c5b-7d2e-8f4a-123456789abc", "canonical"),
        ("018f2c0b6c5b7d2e8f4a123456789abc", "simple"),
        ("urn:uuid:018f2c0b-6c5b-7d2e-8f4a-123456789abc", "urn"),
        ("{018f2c0b-6c5b-7d2e-8f4a-123456789abc}", "braced"),
    ] {
        let output = run(&["inspect", input, "--output", "pretty"]);
        assert!(output.status.success(), "inspection failed for {input}");
        let text = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
        assert!(
            text.lines().any(|line| {
                let mut fields = line.split_whitespace();
                fields.next() == Some("format") && fields.next() == Some(expected)
            }),
            "missing format {expected} for {input}"
        );
    }
}

#[test]
fn pretty_generation_conversion_and_validation_have_command_context() {
    let generated = run(&[
        "generate",
        "v8",
        "--custom",
        "00112233445566778899aabbccddeeff",
        "--output",
        "pretty",
    ]);
    assert!(generated.status.success());
    let generated_text = String::from_utf8(generated.stdout).unwrap();
    assert!(generated_text.contains("Generated UUID"));
    assert!(generated_text.contains("#0 v8 "));
    assert!(!generated_text.lines().any(|line| {
        let mut fields = line.split_whitespace();
        fields.next() == Some("format")
    }));
    assert!(!generated_text.contains("index"));

    let converted = run(&[
        "convert",
        "018f2c0b-6c5b-7d2e-8f4a-123456789abc",
        "--to",
        "urn",
        "--output",
        "pretty",
    ]);
    assert!(converted.status.success());
    let converted_text = String::from_utf8(converted.stdout).unwrap();
    assert!(converted_text.contains("Converted UUID"));
    assert!(!converted_text.lines().any(|line| {
        let mut fields = line.split_whitespace();
        fields.next() == Some("format")
    }));

    let validated = run(&[
        "validate",
        "018f2c0b-6c5b-7d2e-8f4a-123456789abc",
        "--output",
        "pretty",
    ]);
    assert!(validated.status.success());
    assert!(
        String::from_utf8(validated.stdout)
            .unwrap()
            .contains("[ok]")
    );
}

#[test]
fn short_command_aliases_and_conversion_target_work() {
    let inspected = run(&["i", "018f2c0b-6c5b-7d2e-8f4a-123456789abc", "-o", "json"]);
    assert!(inspected.status.success());

    let converted = run(&[
        "c",
        "018f2c0b-6c5b-7d2e-8f4a-123456789abc",
        "-t",
        "simple",
        "-o",
        "plain",
    ]);
    assert!(converted.status.success());
    assert_eq!(
        String::from_utf8(converted.stdout).unwrap().trim(),
        "018f2c0b6c5b7d2e8f4a123456789abc"
    );

    let validated = run(&["v", "018f2c0b-6c5b-7d2e-8f4a-123456789abc", "-o", "json"]);
    assert!(validated.status.success());
}

#[test]
fn stdin_batch_processing_skips_blank_lines() {
    let output = run_with_stdin(
        &["convert", "--to", "urn", "--output", "plain"],
        "\n018f2c0b-6c5b-7d2e-8f4a-123456789abc\n\n",
    );
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout)
            .expect("stdout should be UTF-8")
            .trim(),
        "urn:uuid:018f2c0b-6c5b-7d2e-8f4a-123456789abc"
    );
}

#[test]
fn input_dash_is_rejected_as_a_stdin_sentinel() {
    let output = run(&["convert", "-i", "-", "--to", "simple", "--output", "plain"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8(output.stderr)
            .expect("stderr should be UTF-8")
            .contains("--input accepts a file path")
    );
}

#[test]
fn file_input_and_missing_file_errors_are_distinct() {
    let path = std::env::temp_dir().join(format!("uuidx-cli-test-{}.txt", std::process::id()));
    fs::write(&path, "018f2c0b-6c5b-7d2e-8f4a-123456789abc\n")
        .expect("test input file should be writable");
    let path_string = path.to_string_lossy().into_owned();
    let output = run(&[
        "convert",
        "--input",
        &path_string,
        "--to",
        "simple",
        "--output",
        "plain",
    ]);
    fs::remove_file(&path).expect("test input file should be removable");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        "018f2c0b6c5b7d2e8f4a123456789abc"
    );

    let missing = run(&[
        "inspect",
        "--input",
        "this-file-does-not-exist.uuidx",
        "--output",
        "plain",
    ]);
    assert_eq!(missing.status.code(), Some(3));
    assert!(
        String::from_utf8(missing.stderr)
            .unwrap()
            .contains("input error")
    );
}

#[test]
fn redaction_removes_v6_node_bytes_but_keeps_classification() {
    let generated = run(&[
        "generate",
        "v6",
        "--timestamp",
        "1700000000123",
        "--node",
        "020000000001",
        "--output",
        "plain",
    ]);
    assert!(generated.status.success());
    let uuid = String::from_utf8(generated.stdout)
        .unwrap()
        .trim()
        .to_owned();
    let inspected = run(&["inspect", &uuid, "--output", "json", "--redact-sensitive"]);
    assert!(inspected.status.success());
    let record: Value = serde_json::from_slice(&inspected.stdout).expect("JSON should parse");
    assert!(record["metadata"]["node_id"].is_null());
    assert_eq!(
        record["metadata"]["node_kind"],
        "unicast / may be hardware-derived"
    );
}

#[test]
fn help_lists_the_user_facing_commands() {
    let output = run(&["--help"]);
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).expect("help should be UTF-8");
    assert!(help.contains("generate"));
    assert!(help.contains("inspect"));
    assert!(help.contains("validate"));
    assert!(help.contains("convert"));
    assert!(help.contains("generate  Generate UUID v3-v8 values [alias: g]"));
    assert!(help.contains("inspect   Inspect UUID, ULID, NanoID, and Snowflake values [alias: i]"));
    assert!(help.contains("validate  Validate UUID syntax and version structure [alias: v]"));
    assert!(
        help.contains("convert   Convert UUID text between standard textual formats [alias: c]")
    );
    let root_usage = if cfg!(windows) {
        "Usage: uuidx.exe [OPTIONS] <COMMAND>"
    } else {
        "Usage: uuidx [OPTIONS] <COMMAND>"
    };
    assert!(help.contains(root_usage));
    assert!(help.contains("Application:"));
    assert!(help.contains("Global options:"));
    assert!(help.contains("A modern CLI for UUID generation, inspection, and conversion."));
}

#[test]
fn command_help_explains_scoped_options_and_formats() {
    let inspect = run(&["inspect", "--help"]);
    assert!(inspect.status.success());
    let inspect_help = String::from_utf8(inspect.stdout).expect("help should be UTF-8");
    assert!(inspect_help.contains("Input options:"));
    assert!(inspect_help.contains("Inspection options:"));
    assert!(inspect_help.contains("--layout"));
    assert!(inspect_help.contains("--redact-sensitive"));
    assert!(inspect_help.contains("--fail-fast"));

    let convert = run(&["convert", "--help"]);
    assert!(convert.status.success());
    let convert_help = String::from_utf8(convert.stdout).expect("help should be UTF-8");
    assert!(convert_help.contains("Conversion options:"));
    let convert_usage = if cfg!(windows) {
        "Usage: uuidx.exe convert [OPTIONS] [VALUE]..."
    } else {
        "Usage: uuidx convert [OPTIONS] [VALUE]..."
    };
    assert!(convert_help.contains(convert_usage));
    assert!(convert_help.contains("[default: canonical]"));
    assert!(convert_help.contains("canonical"));
    assert!(convert_help.contains("simple"));
    assert!(convert_help.contains("urn"));
    assert!(convert_help.contains("braced"));

    let generate = run(&["generate", "--help"]);
    assert!(generate.status.success());
    let generate_help = String::from_utf8(generate.stdout).expect("help should be UTF-8");
    assert!(generate_help.contains("[v3/v5] Namespace UUID"));
    assert!(generate_help.contains("[v3/v5] Name bytes"));
    assert!(generate_help.contains("[v6] Six-byte node ID"));
    assert!(generate_help.contains("[v6/v7] RFC3339 timestamp"));
    assert!(generate_help.contains("[v8] Sixteen custom bytes as hexadecimal"));
}

#[cfg(feature = "ulid-inspect")]
#[test]
fn inspect_recognizes_ulid_without_annotations() {
    let output = run(&["inspect", "01ARZ3NDEKTSV4RRFFQ69G5FAV", "--output", "json"]);
    assert!(output.status.success());
    let record: Value = serde_json::from_slice(&output.stdout).expect("ULID JSON should parse");
    assert_eq!(record["kind"], "ulid");
    assert_eq!(record["metadata"]["type"], "ulid");
    assert!(record["warnings"].is_null());
}

#[test]
fn inspect_recognizes_nanoid_and_snowflake_without_annotations() {
    let nanoid = run(&["inspect", "V1StGXR8_Z5jdHi6B-myT", "--output", "json"]);
    assert!(nanoid.status.success());
    let record: Value = serde_json::from_slice(&nanoid.stdout).expect("NanoID JSON should parse");
    assert_eq!(record["kind"], "nanoid");
    assert_eq!(record["metadata"]["length"], 21);
    assert_eq!(record["metadata"]["entropy_bits"], 126);
    assert!(record["warnings"].is_null());

    let snowflake = run(&["inspect", "1724552287438348288", "--output", "json"]);
    assert!(snowflake.status.success());
    let record: Value =
        serde_json::from_slice(&snowflake.stdout).expect("Snowflake JSON should parse");
    assert_eq!(record["kind"], "snowflake");
    assert_eq!(record["metadata"]["epoch"], "twitter");
    assert!(record["metadata"]["timestamp_ms"].is_number());
    assert!(record["warnings"].is_null());
}

#[test]
fn inspect_layout_adds_details_for_non_uuid_identifiers() {
    let cases = [
        ("V1StGXR8_Z5jdHi6B-myT", "entropy_bits", None),
        ("1724552287438348288", "datacenter_id", Some("Bit layout")),
    ];

    for (input, detail, layout) in cases {
        let summary = run(&["inspect", input, "--output", "pretty"]);
        assert!(summary.status.success());
        let summary = String::from_utf8(summary.stdout).unwrap();
        assert!(!summary.contains(detail));
        assert!(!summary.contains("profile"));
        assert!(!summary.contains("status"));

        let detailed = run(&["inspect", input, "--output", "pretty", "--layout"]);
        assert!(detailed.status.success());
        let detailed = String::from_utf8(detailed.stdout).unwrap();
        assert!(detailed.contains(detail));
        if let Some(layout) = layout {
            assert!(detailed.contains(layout));
        }
    }
}

#[test]
fn inspect_rejects_removed_kind_option() {
    let output = run(&[
        "inspect",
        "018f2c0b-6c5b-7d2e-8f4a-123456789abc",
        "--kind",
        "uuid",
    ]);
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("unexpected argument '--kind'")
    );
}

#[test]
fn inspect_type_forces_the_requested_family() {
    let ok = run(&[
        "inspect",
        "V1StGXR8_Z5jdHi6B-myT",
        "--type",
        "nanoid",
        "--output",
        "json",
    ]);
    assert!(ok.status.success());
    let record: Value = serde_json::from_slice(&ok.stdout).expect("JSON should parse");
    assert_eq!(record["kind"], "nanoid");

    let mismatch = run(&[
        "inspect",
        "V1StGXR8_Z5jdHi6B-myT",
        "--type",
        "uuid",
        "--output",
        "json",
    ]);
    assert_eq!(mismatch.status.code(), Some(1));
    let record: Value = serde_json::from_slice(&mismatch.stdout).expect("JSON should parse");
    assert_eq!(record["ok"], false);
    assert_eq!(record["error"]["code"], "invalid_identifier");
}

#[test]
fn inspect_type_rejects_an_unsupported_family() {
    let output = run(&["inspect", "x", "--type", "objectid"]);
    assert_eq!(output.status.code(), Some(2));
}

#[cfg(feature = "ulid-inspect")]
#[test]
fn inspect_type_accepts_ulid() {
    let ok = run(&[
        "inspect",
        "01ARZ3NDEKTSV4RRFFQ69G5FAV",
        "--type",
        "ulid",
        "--output",
        "json",
    ]);
    assert!(ok.status.success());
    let record: Value = serde_json::from_slice(&ok.stdout).expect("JSON should parse");
    assert_eq!(record["kind"], "ulid");
}

#[test]
fn validate_type_validates_other_families_and_tags_kind() {
    let ok = run(&[
        "validate",
        "V1StGXR8_Z5jdHi6B-myT",
        "--type",
        "nanoid",
        "--output",
        "json",
    ]);
    assert!(ok.status.success());
    let record: Value = serde_json::from_slice(&ok.stdout).expect("JSON should parse");
    assert_eq!(record["operation"], "validate");
    assert_eq!(record["kind"], "nanoid");
    assert!(record["bytes"].is_null());

    let uuid_ok = run(&[
        "validate",
        "018f2c0b-6c5b-7d2e-8f4a-123456789abc",
        "--type",
        "uuid",
        "--output",
        "json",
    ]);
    assert!(uuid_ok.status.success());
    let record: Value = serde_json::from_slice(&uuid_ok.stdout).expect("JSON should parse");
    assert_eq!(record["kind"], "uuid");
    assert_eq!(record["version"], "v7");
    assert_eq!(record["bytes"].as_str().map(str::len), Some(32));

    let mismatch = run(&[
        "validate",
        "018f2c0b-6c5b-7d2e-8f4a-123456789abc",
        "--type",
        "nanoid",
        "--output",
        "json",
    ]);
    assert_eq!(mismatch.status.code(), Some(1));
    let record: Value = serde_json::from_slice(&mismatch.stdout).expect("JSON should parse");
    assert_eq!(record["error"]["code"], "invalid_identifier");

    let default_uuid = run(&["validate", "not-a-uuid", "--output", "json"]);
    assert_eq!(default_uuid.status.code(), Some(1));
    let record: Value = serde_json::from_slice(&default_uuid.stdout).expect("JSON should parse");
    assert_eq!(record["error"]["code"], "invalid_uuid");
}

#[test]
fn convert_applies_requested_case_to_uuid_digits() {
    let uuid = "018f2c0b-6c5b-7d2e-8f4a-123456789abc";

    let lower = run(&[
        "convert", uuid, "--to", "urn", "--case", "lower", "--output", "plain",
    ]);
    assert!(lower.status.success());
    assert_eq!(
        String::from_utf8(lower.stdout).unwrap().trim(),
        format!("urn:uuid:{uuid}")
    );

    let cases = [
        ("canonical", "018F2C0B-6C5B-7D2E-8F4A-123456789ABC"),
        ("simple", "018F2C0B6C5B7D2E8F4A123456789ABC"),
        ("urn", "urn:uuid:018F2C0B-6C5B-7D2E-8F4A-123456789ABC"),
        ("braced", "{018F2C0B-6C5B-7D2E-8F4A-123456789ABC}"),
    ];
    for (format, expected) in cases {
        let output = run(&[
            "convert", uuid, "--to", format, "--case", "upper", "--output", "plain",
        ]);
        assert!(output.status.success(), "convert {format} failed");
        assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), expected);
    }
}

#[test]
fn generate_case_changes_value_but_not_bytes() {
    let upper = run(&[
        "generate", "v4", "--format", "simple", "--case", "upper", "--output", "json",
    ]);
    assert!(upper.status.success());
    let record: Value = serde_json::from_slice(&upper.stdout).expect("JSON should parse");
    let value = record["value"].as_str().expect("value should be a string");
    let bytes = record["bytes"].as_str().expect("bytes should be a string");
    assert_eq!(value, bytes.to_ascii_uppercase());
    assert_eq!(bytes, bytes.to_ascii_lowercase());

    let lower = run(&[
        "generate", "v4", "--format", "simple", "--case", "lower", "--output", "json",
    ]);
    assert!(lower.status.success());
    let record: Value = serde_json::from_slice(&lower.stdout).expect("JSON should parse");
    assert_eq!(record["value"], record["bytes"]);
}

#[test]
fn scoped_help_lists_type_and_case_options() {
    for command in ["inspect", "validate"] {
        let output = run(&[command, "--help"]);
        assert!(output.status.success());
        let help = String::from_utf8(output.stdout).expect("help should be UTF-8");
        assert!(help.contains("--type"), "{command} help should list --type");
    }

    for command in ["generate", "convert"] {
        let output = run(&[command, "--help"]);
        assert!(output.status.success());
        let help = String::from_utf8(output.stdout).expect("help should be UTF-8");
        assert!(help.contains("--case"), "{command} help should list --case");
    }
}
