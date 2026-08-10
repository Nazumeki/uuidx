use clap::Args;

use super::{InputArgs, UuidFormatArg};

#[cfg(windows)]
const CONVERT_USAGE: &str = "uuidx.exe convert [OPTIONS] [VALUE]...";
#[cfg(not(windows))]
const CONVERT_USAGE: &str = "uuidx convert [OPTIONS] [VALUE]...";

#[derive(Debug, Args)]
#[command(
    next_help_heading = "Conversion options",
    override_usage = CONVERT_USAGE
)]
pub struct ConvertArgs {
    #[command(flatten)]
    pub input: InputArgs,

    /// UUID output format.
    #[arg(
        short = 't',
        long,
        value_name = "FORMAT",
        value_enum,
        default_value_t = UuidFormatArg::Canonical,
        help_heading = "Conversion options"
    )]
    pub to: UuidFormatArg,
}
