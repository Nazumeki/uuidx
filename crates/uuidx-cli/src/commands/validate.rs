use crate::{cli::ValidateArgs, errors::CliError, input, output::Output};

pub fn run(args: &ValidateArgs, output: &mut Output) -> Result<bool, CliError> {
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
