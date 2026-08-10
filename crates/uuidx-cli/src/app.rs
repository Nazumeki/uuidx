use crate::{
    cli::{Cli, Command},
    commands,
    errors::AppResult,
    output::Output,
};

pub fn run(cli: Cli) -> AppResult {
    let mut output = Output::new(&cli.global);
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
