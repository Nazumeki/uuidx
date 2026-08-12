use crate::{
    cli::{InspectArgs, InspectKindArg},
    errors::CliError,
    input,
    output::{Output, OutputWriter},
};

pub fn run<WOut, WErr>(
    args: &InspectArgs,
    output: &mut Output<WOut, WErr>,
) -> Result<bool, CliError>
where
    WOut: OutputWriter,
    WErr: OutputWriter,
{
    let fail_fast = args.input.fail_fast;
    let kind = args.kind;
    let redact_sensitive = args.redact_sensitive;
    let show_layout = args.layout;
    let summary =
        input::for_each_record(&args.input, fail_fast, |index, value| {
            match inspect_value(kind, value) {
                Ok(InspectionValue::Uuid(uuid)) => {
                    output.inspected_uuid(index, value, &uuid, redact_sensitive, show_layout)?;
                    Ok(false)
                }
                #[cfg(feature = "ulid-inspect")]
                Ok(InspectionValue::Ulid(ulid)) => {
                    output.inspected_ulid(index, value, &ulid)?;
                    Ok(false)
                }
                Err(error) => {
                    output.record_error("inspect", index, value, "invalid_identifier", &error)?;
                    Ok(true)
                }
            }
        })?;
    Ok(summary.data_errors)
}

enum InspectionValue {
    Uuid(uuidx_core::UuidInspection),
    #[cfg(feature = "ulid-inspect")]
    Ulid(uuidx_core::UlidInspection),
}

fn inspect_value(kind: InspectKindArg, value: &str) -> Result<InspectionValue, String> {
    match kind {
        InspectKindArg::Uuid => uuidx_core::parse_uuid(value)
            .map(|uuid| InspectionValue::Uuid(uuidx_core::inspect_uuid(&uuid)))
            .map_err(|error| error.to_string()),
        #[cfg(feature = "ulid-inspect")]
        InspectKindArg::Ulid => uuidx_core::inspect_ulid(value)
            .map(InspectionValue::Ulid)
            .map_err(|error| error.to_string()),
        #[cfg(not(feature = "ulid-inspect"))]
        InspectKindArg::Ulid => {
            Err("ULID inspection support was disabled at build time".to_owned())
        }
        InspectKindArg::Auto => {
            if let Ok(uuid) = uuidx_core::parse_uuid(value) {
                return Ok(InspectionValue::Uuid(uuidx_core::inspect_uuid(&uuid)));
            }
            #[cfg(feature = "ulid-inspect")]
            if let Ok(ulid) = uuidx_core::inspect_ulid(value) {
                return Ok(InspectionValue::Ulid(ulid));
            }
            Err("input is neither a valid UUID nor a supported ULID".to_owned())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{GlobalOptions, InputArgs, OutputModeArg};

    const UUID: &str = "018f2c0b-6c5b-7d2e-8f4a-123456789abc";

    #[test]
    fn inspect_value_supports_uuid_modes_and_rejects_invalid_input() {
        assert!(matches!(
            inspect_value(InspectKindArg::Uuid, UUID),
            Ok(InspectionValue::Uuid(_))
        ));
        assert!(matches!(
            inspect_value(InspectKindArg::Auto, UUID),
            Ok(InspectionValue::Uuid(_))
        ));
        assert!(inspect_value(InspectKindArg::Uuid, "not-a-uuid").is_err());
        assert!(inspect_value(InspectKindArg::Auto, "not-an-identifier").is_err());
    }

    #[cfg(feature = "ulid-inspect")]
    #[test]
    fn inspect_value_supports_ulid_mode() {
        assert!(matches!(
            inspect_value(InspectKindArg::Ulid, "01ARZ3NDEKTSV4RRFFQ69G5FAV"),
            Ok(InspectionValue::Ulid(_))
        ));
        assert!(matches!(
            inspect_value(InspectKindArg::Auto, "01ARZ3NDEKTSV4RRFFQ69G5FAV"),
            Ok(InspectionValue::Ulid(_))
        ));
        assert!(inspect_value(InspectKindArg::Ulid, "not-a-ulid").is_err());
    }

    #[cfg(not(feature = "ulid-inspect"))]
    #[test]
    fn inspect_value_explains_disabled_ulid_support() {
        assert!(matches!(
            inspect_value(InspectKindArg::Ulid, "01ARZ3NDEKTSV4RRFFQ69G5FAV"),
            Err(message) if message.contains("disabled")
        ));
    }

    #[test]
    fn run_marks_invalid_records_as_data_errors() {
        let args = InspectArgs {
            input: InputArgs {
                values: vec![UUID.to_owned(), "not-an-identifier".to_owned()],
                input: None,
                fail_fast: false,
            },
            kind: InspectKindArg::Auto,
            layout: false,
            redact_sensitive: false,
        };
        let mut output = Output::with_writers(
            &GlobalOptions {
                output: OutputModeArg::Json,
            },
            Vec::new(),
            Vec::new(),
        );
        assert!(run(&args, &mut output).unwrap());
    }

    #[test]
    fn run_propagates_success_and_error_renderer_failures() {
        let valid = InspectArgs {
            input: InputArgs {
                values: vec![UUID.to_owned()],
                input: None,
                fail_fast: false,
            },
            kind: InspectKindArg::Uuid,
            layout: false,
            redact_sensitive: false,
        };
        let mut output = crate::output::test_output(
            OutputModeArg::Plain,
            crate::output::TestWriter::failing_write(),
            crate::output::TestWriter::working(),
        );
        assert!(matches!(run(&valid, &mut output), Err(CliError::Output(_))));

        let invalid = InspectArgs {
            input: InputArgs {
                values: vec!["not-an-identifier".to_owned()],
                input: None,
                fail_fast: false,
            },
            kind: InspectKindArg::Auto,
            layout: false,
            redact_sensitive: false,
        };
        let mut output = crate::output::test_output(
            OutputModeArg::Plain,
            crate::output::TestWriter::working(),
            crate::output::TestWriter::failing_write(),
        );
        assert!(matches!(
            run(&invalid, &mut output),
            Err(CliError::Output(_))
        ));

        #[cfg(feature = "ulid-inspect")]
        {
            let ulid = InspectArgs {
                input: InputArgs {
                    values: vec!["01ARZ3NDEKTSV4RRFFQ69G5FAV".to_owned()],
                    input: None,
                    fail_fast: false,
                },
                kind: InspectKindArg::Ulid,
                layout: false,
                redact_sensitive: false,
            };
            let mut output = crate::output::test_output(
                OutputModeArg::Plain,
                crate::output::TestWriter::failing_write(),
                crate::output::TestWriter::working(),
            );
            assert!(matches!(run(&ulid, &mut output), Err(CliError::Output(_))));
        }
    }
}
