use crate::{cli::ConvertArgs, errors::CliError, input, output::Output};
use uuidx_core::UuidOutputFormat;

pub fn run(args: &ConvertArgs, output: &mut Output) -> Result<bool, CliError> {
    let format: UuidOutputFormat = args.to.into();
    let summary = input::for_each_record(&args.input, args.input.fail_fast, |index, value| {
        match uuidx_core::parse_uuid(value) {
            Ok(uuid) => {
                let converted = uuidx_core::format_uuid(&uuid, format);
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
