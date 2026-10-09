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
    let family = args.kind.map(Into::into);
    let summary = input::for_each_record(&args.input, fail_fast, |index, value| {
        let inspection = match family {
            Some(family) => {
                uuidx_core::inspect_identifier_as(value, family).map_err(|error| error.to_string())
            }
            None => uuidx_core::inspect_identifier(value).map_err(|error| error.to_string()),
        };
        match inspection {
            Ok(uuidx_core::IdentifierInspection::Uuid(uuid)) => {
                output.inspected_uuid(index, value, &uuid, redact_sensitive, show_layout)?;
                Ok(false)
            }
            #[cfg(feature = "ulid-inspect")]
            Ok(uuidx_core::IdentifierInspection::Ulid(ulid)) => {
                output.inspected_ulid(index, value, &ulid, show_layout)?;
                Ok(false)
            }
            Ok(uuidx_core::IdentifierInspection::Nanoid(nanoid)) => {
                output.inspected_nanoid(index, value, &nanoid, show_layout)?;
                Ok(false)
            }
            Ok(uuidx_core::IdentifierInspection::Snowflake(snowflake)) => {
                output.inspected_snowflake(index, value, &snowflake, show_layout)?;
                Ok(false)
            }
            Err(message) => {
                output.record_error("inspect", index, value, "invalid_identifier", &message)?;
                Ok(true)
            }
        }
    })?;
    Ok(summary.data_errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{GlobalOptions, InputArgs, OutputModeArg};

    const UUID: &str = "018f2c0b-6c5b-7d2e-8f4a-123456789abc";

    #[test]
    fn inspect_value_recognizes_uuids_and_rejects_invalid_input() {
        assert!(matches!(
            uuidx_core::inspect_identifier(UUID),
            Ok(uuidx_core::IdentifierInspection::Uuid(_))
        ));
        assert!(uuidx_core::inspect_identifier("not-an-identifier").is_err());
    }

    #[cfg(feature = "ulid-inspect")]
    #[test]
    fn inspect_value_recognizes_ulids() {
        assert!(matches!(
            uuidx_core::inspect_identifier("01ARZ3NDEKTSV4RRFFQ69G5FAV"),
            Ok(uuidx_core::IdentifierInspection::Ulid(_))
        ));
    }

    #[test]
    fn inspect_value_recognizes_nanoids_and_snowflakes() {
        assert!(matches!(
            uuidx_core::inspect_identifier("V1StGXR8_Z5jdHi6B-myT"),
            Ok(uuidx_core::IdentifierInspection::Nanoid(_))
        ));
        assert!(matches!(
            uuidx_core::inspect_identifier("1724552287438348288"),
            Ok(uuidx_core::IdentifierInspection::Snowflake(_))
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
            kind: None,
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
            kind: None,
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
            kind: None,
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
                kind: None,
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
