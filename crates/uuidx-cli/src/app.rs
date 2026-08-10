use crate::{
    cli::{Cli, Command},
    commands,
    errors::AppResult,
    output::{Output, OutputWriter},
};

pub fn run(cli: Cli) -> AppResult {
    let output = Output::new(&cli.global);
    run_with_output(cli, output)
}

fn run_with_output<WOut, WErr>(cli: Cli, mut output: Output<WOut, WErr>) -> AppResult
where
    WOut: OutputWriter,
    WErr: OutputWriter,
{
    let result = match cli.command {
        Command::Generate(args) => commands::generate::run(&args, &mut output),
        Command::Inspect(args) => commands::inspect::run(&args, &mut output),
        Command::Validate(args) => commands::validate::run(&args, &mut output),
        Command::Convert(args) => commands::convert::run(&args, &mut output),
    };

    let result = match result {
        Ok(true) => AppResult::DataErrors,
        Ok(false) => AppResult::Success,
        Err(error) => {
            let _ = output.top_level_error(&error.to_string());
            AppResult::Failure(error)
        }
    };

    if let Err(error) = output.flush() {
        return AppResult::Failure(error);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{
        Command, ConvertArgs, GenerateArgs, GlobalOptions, InputArgs, InspectArgs, InspectKindArg,
        OutputModeArg, UuidFormatArg, ValidateArgs,
    };

    const UUID: &str = "018f2c0b-6c5b-7d2e-8f4a-123456789abc";

    fn cli(command: Command) -> Cli {
        Cli {
            version: None,
            global: GlobalOptions {
                output: OutputModeArg::Json,
            },
            command,
        }
    }

    fn input(values: &[&str]) -> InputArgs {
        InputArgs {
            values: values.iter().map(|value| (*value).to_owned()).collect(),
            input: None,
            fail_fast: false,
        }
    }

    fn run_silent(cli: Cli) -> AppResult {
        let output = Output::with_writers(&cli.global, Vec::new(), Vec::new());
        run_with_output(cli, output)
    }

    #[test]
    fn dispatches_each_command_and_maps_data_and_usage_failures() {
        assert!(matches!(
            run_silent(cli(Command::Generate(GenerateArgs {
                target: "v4".to_owned(),
                count: 1,
                namespace: None,
                name: None,
                node: None,
                timestamp: None,
                custom: None,
                format: UuidFormatArg::Canonical,
            }))),
            AppResult::Success
        ));
        assert!(matches!(
            run_silent(cli(Command::Inspect(InspectArgs {
                input: input(&[UUID]),
                kind: InspectKindArg::Uuid,
                layout: false,
                redact_sensitive: false,
            }))),
            AppResult::Success
        ));
        assert!(matches!(
            run_silent(cli(Command::Convert(ConvertArgs {
                input: input(&[UUID]),
                to: UuidFormatArg::Simple,
            }))),
            AppResult::Success
        ));
        assert!(matches!(
            run_silent(cli(Command::Validate(ValidateArgs {
                input: input(&[UUID]),
            }))),
            AppResult::Success
        ));
        assert!(matches!(
            run_silent(cli(Command::Validate(ValidateArgs {
                input: input(&["not-a-uuid"]),
            }))),
            AppResult::DataErrors
        ));
        assert!(matches!(
            run_silent(cli(Command::Generate(GenerateArgs {
                target: "v1".to_owned(),
                count: 1,
                namespace: None,
                name: None,
                node: None,
                timestamp: None,
                custom: None,
                format: UuidFormatArg::Canonical,
            }))),
            AppResult::Failure(_)
        ));
    }

    #[test]
    fn public_run_uses_the_real_output_boundary() {
        let mut cli = cli(Command::Validate(ValidateArgs {
            input: input(&[UUID]),
        }));
        cli.global.output = OutputModeArg::Plain;
        assert!(matches!(run(cli), AppResult::Success));
    }
}
