use std::path::PathBuf;

use clap::{
    ArgAction, Args, Parser, Subcommand, ValueEnum,
    builder::styling::{AnsiColor, Styles},
};

use super::{ConvertArgs, GenerateArgs, InspectArgs, ValidateArgs};

const CLI_STYLES: Styles = Styles::styled()
    .header(AnsiColor::Cyan.on_default().bold())
    .usage(AnsiColor::Green.on_default().bold())
    .literal(AnsiColor::Blue.on_default().bold())
    .placeholder(AnsiColor::Yellow.on_default())
    .error(AnsiColor::Red.on_default().bold())
    .valid(AnsiColor::Green.on_default())
    .invalid(AnsiColor::Red.on_default())
    .context(AnsiColor::BrightBlack.on_default())
    .context_value(AnsiColor::Cyan.on_default());

#[cfg(windows)]
const ROOT_USAGE: &str = "uuidx.exe [OPTIONS] <COMMAND>";
#[cfg(not(windows))]
const ROOT_USAGE: &str = "uuidx [OPTIONS] <COMMAND>";

#[derive(Debug, Parser)]
#[command(
    name = "uuidx",
    version,
    about = "A modern CLI for UUID generation, inspection, and conversion.",
    styles = CLI_STYLES,
    override_usage = ROOT_USAGE,
    disable_version_flag = true,
    subcommand_required = true,
    arg_required_else_help = true,
    disable_help_subcommand = true
)]
pub struct Cli {
    /// Print the application version.
    // ArgAction::Version exits before producing a value; Option keeps derive extraction optional.
    #[arg(
        short = 'v',
        long = "version",
        help_heading = "Application",
        action = ArgAction::Version
    )]
    pub version: Option<bool>,

    #[command(flatten)]
    pub global: GlobalOptions,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Args)]
pub struct GlobalOptions {
    /// Human output mode. Auto is pretty on a terminal and plain in a pipe.
    #[arg(
        short = 'o',
        long,
        global = true,
        help_heading = "Global options",
        value_enum,
        default_value_t = OutputModeArg::Auto
    )]
    pub output: OutputModeArg,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputModeArg {
    Auto,
    Pretty,
    Plain,
    Json,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Generate UUID v3-v8 values.
    #[command(visible_alias = "g")]
    Generate(GenerateArgs),
    /// Inspect UUID, ULID, NanoID, and Snowflake values.
    #[command(visible_alias = "i")]
    Inspect(InspectArgs),
    /// Validate UUID syntax and version structure.
    #[command(visible_alias = "v")]
    Validate(ValidateArgs),
    /// Convert UUID text between standard textual formats.
    #[command(visible_alias = "c")]
    Convert(ConvertArgs),
}

#[derive(Debug, Args)]
pub struct InputArgs {
    /// Values to process. Without values, input is read from stdin.
    #[arg(
        value_name = "VALUE",
        conflicts_with = "input",
        help_heading = "Input options"
    )]
    pub values: Vec<String>,

    /// Read one value per line from a file path. Omit input options to read piped stdin.
    #[arg(
        short = 'i',
        long,
        value_name = "FILE",
        value_parser = file_path,
        conflicts_with = "values",
        help_heading = "Input options"
    )]
    pub input: Option<PathBuf>,

    /// Stop after the first invalid input record.
    #[arg(short = 'f', long, help_heading = "Input options")]
    pub fail_fast: bool,
}

fn file_path(value: &str) -> Result<PathBuf, String> {
    if value == "-" {
        return Err("--input accepts a file path; omit --input to read piped stdin".to_owned());
    }
    Ok(PathBuf::from(value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::UuidFormatArg;

    #[test]
    fn parser_preserves_aliases_and_generation_defaults() {
        let cli = Cli::try_parse_from(["uuidx", "g"]).expect("alias should parse");
        assert_eq!(cli.global.output, OutputModeArg::Auto);
        match cli.command {
            Command::Generate(args) => {
                assert_eq!(args.target, "v7");
                assert_eq!(args.count, 1);
            }
            command => panic!("expected generate command, got {command:?}"),
        }

        let cli = Cli::try_parse_from(["uuidx", "convert", "-o", "json", "--to", "urn"])
            .expect("global and command options should parse");
        assert_eq!(cli.global.output, OutputModeArg::Json);
        assert!(matches!(
            cli.command,
            Command::Convert(args) if args.to == UuidFormatArg::Urn
        ));
    }

    #[test]
    fn file_path_rejects_stdin_sentinel_and_keeps_paths() {
        assert_eq!(file_path("values.uuid"), Ok(PathBuf::from("values.uuid")));
        assert!(file_path("-").is_err());
    }
}
