use crate::{
    cli::ConvertArgs,
    errors::CliError,
    input,
    output::{Output, OutputWriter},
};
use uuidx_core::UuidOutputFormat;

pub fn run<WOut, WErr>(
    args: &ConvertArgs,
    output: &mut Output<WOut, WErr>,
) -> Result<bool, CliError>
where
    WOut: OutputWriter,
    WErr: OutputWriter,
{
    let format: UuidOutputFormat = args.to.into();
    let case = args.case.into();
    let summary = input::for_each_record(&args.input, args.input.fail_fast, |index, value| {
        match uuidx_core::parse_uuid(value) {
            Ok(uuid) => {
                let converted = uuidx_core::format_uuid_with_case(&uuid, format, case);
                output.converted(index, value, &uuid, &converted, format)?;
                Ok(false)
            }
            Err(error) => {
                output.record_error("convert", index, value, "invalid_uuid", &error.to_string())?;
                Ok(true)
            }
        }
    })?;
    Ok(summary.data_errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{GlobalOptions, InputArgs, OutputModeArg, TextCaseArg, UuidFormatArg};

    fn args(values: &[&str]) -> ConvertArgs {
        ConvertArgs {
            input: InputArgs {
                values: values.iter().map(|value| (*value).to_owned()).collect(),
                input: None,
                fail_fast: false,
            },
            to: UuidFormatArg::Urn,
            case: TextCaseArg::Lower,
        }
    }

    fn output() -> Output<Vec<u8>, Vec<u8>> {
        Output::with_writers(
            &GlobalOptions {
                output: OutputModeArg::Json,
            },
            Vec::new(),
            Vec::new(),
        )
    }

    #[test]
    fn run_converts_valid_records_and_marks_invalid_records() {
        let mut valid_output = output();
        assert!(
            !run(
                &args(&["018f2c0b-6c5b-7d2e-8f4a-123456789abc"]),
                &mut valid_output
            )
            .unwrap()
        );

        let mut invalid_output = output();
        assert!(run(&args(&["not-a-uuid"]), &mut invalid_output).unwrap());
    }

    #[test]
    fn run_propagates_success_and_error_renderer_failures() {
        let mut success_output = crate::output::test_output(
            OutputModeArg::Plain,
            crate::output::TestWriter::failing_write(),
            crate::output::TestWriter::working(),
        );
        assert!(matches!(
            run(
                &args(&["018f2c0b-6c5b-7d2e-8f4a-123456789abc"]),
                &mut success_output
            ),
            Err(CliError::Output(_))
        ));

        let mut error_output = crate::output::test_output(
            OutputModeArg::Plain,
            crate::output::TestWriter::working(),
            crate::output::TestWriter::failing_write(),
        );
        assert!(matches!(
            run(&args(&["not-a-uuid"]), &mut error_output),
            Err(CliError::Output(_))
        ));
    }
}
