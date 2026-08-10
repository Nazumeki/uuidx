use clap::{Args, ValueEnum};

use super::InputArgs;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum InspectKindArg {
    Auto,
    Uuid,
    Ulid,
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Inspection options")]
pub struct InspectArgs {
    #[command(flatten)]
    pub input: InputArgs,

    /// Input family. Auto recognizes UUID first and then optional ULID input.
    #[arg(
        short = 'k',
        long,
        value_enum,
        default_value_t = InspectKindArg::Auto,
        help_heading = "Inspection options"
    )]
    pub kind: InspectKindArg,

    /// Show bit offsets and values in pretty output.
    #[arg(short = 'L', long, help_heading = "Inspection options")]
    pub layout: bool,

    /// Omit time-based UUID node IDs while keeping their classification.
    #[arg(short = 'r', long, help_heading = "Inspection options")]
    pub redact_sensitive: bool,
}
