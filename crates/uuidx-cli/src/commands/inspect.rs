use crate::{
    cli::InspectArgs,
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
    let redact_sensitive = args.redact_sensitive;
    let show_layout = args.layout;
    let summary =
        input::for_each_record(&args.input, fail_fast, |index, value| {
            match inspect_value(value) {
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

fn inspect_value(value: &str) -> Result<InspectionValue, String> {
    if let Ok(uuid) = uuidx_core::parse_uuid(value) {
        return Ok(InspectionValue::Uuid(uuidx_core::inspect_uuid(&uuid)));
    }
    #[cfg(feature = "ulid-inspect")]
    if let Ok(ulid) = uuidx_core::inspect_ulid(value) {
        return Ok(InspectionValue::Ulid(ulid));
    }
    Err("input is neither a valid UUID nor a supported ULID".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{GlobalOptions, InputArgs, OutputModeArg};

    const UUID: &str = "018f2c0b-6c5b-7d2e-8f4a-123456789abc";

    #[test]
    fn inspect_value_recognizes_uuids_and_rejects_invalid_input() {
        assert!(matches!(inspect_value(UUID), Ok(InspectionValue::Uuid(_))));
        assert!(inspect_value("not-an-identifier").is_err());
    }

    #[cfg(feature = "ulid-inspect")]
    #[test]
    fn inspect_value_recognizes_ulids() {
        assert!(matches!(
            inspect_value("01ARZ3NDEKTSV4RRFFQ69G5FAV"),
            Ok(InspectionValue::Ulid(_))
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
