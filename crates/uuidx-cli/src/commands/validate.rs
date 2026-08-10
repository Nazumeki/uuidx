use crate::{
    cli::ValidateArgs,
    errors::CliError,
    input,
    output::{Output, OutputWriter},
};

pub fn run<WOut, WErr>(
    args: &ValidateArgs,
    output: &mut Output<WOut, WErr>,
) -> Result<bool, CliError>
where
    WOut: OutputWriter,
    WErr: OutputWriter,
{
    let summary = input::for_each_record(&args.input, args.input.fail_fast, |index, value| {
        match uuidx_core::parse_uuid(value) {
            Ok(uuid) => {
                output.validated(index, value, &uuid)?;
                Ok(false)
            }
            Err(error) => {
                output.record_error(
                    "validate",
                    index,
                    value,
                    "invalid_uuid",
                    &error.to_string(),
                )?;
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

    fn args(values: &[&str]) -> ValidateArgs {
        ValidateArgs {
            input: InputArgs {
                values: values.iter().map(|value| (*value).to_owned()).collect(),
                input: None,
                fail_fast: false,
            },
        }
    }

    #[test]
    fn run_validates_records_and_returns_data_error_status() {
        let mut output = Output::with_writers(
            &GlobalOptions {
                output: OutputModeArg::Json,
            },
            Vec::new(),
            Vec::new(),
        );
        assert!(
            !run(
                &args(&["018f2c0b-6c5b-7d2e-8f4a-123456789abc"]),
                &mut output
            )
            .unwrap()
        );

        let mut output = Output::with_writers(
            &GlobalOptions {
                output: OutputModeArg::Json,
            },
            Vec::new(),
            Vec::new(),
        );
        assert!(run(&args(&["not-a-uuid"]), &mut output).unwrap());
    }
}
