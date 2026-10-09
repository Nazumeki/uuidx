use clap::Args;

use super::{IdentifierTypeArg, InputArgs};

#[derive(Debug, Args)]
#[command(next_help_heading = "Inspection options")]
pub struct InspectArgs {
    #[command(flatten)]
    pub input: InputArgs,

    /// Force one identifier family instead of automatic detection.
    #[arg(
        short = 't',
        long = "type",
        value_enum,
        value_name = "TYPE",
        help_heading = "Inspection options"
    )]
    pub kind: Option<IdentifierTypeArg>,

    /// Show decoded fields and bit-layout details in pretty output.
    #[arg(short = 'L', long, help_heading = "Inspection options")]
    pub layout: bool,

    /// Omit time-based UUID node IDs while keeping their classification.
    #[arg(short = 'r', long, help_heading = "Inspection options")]
    pub redact_sensitive: bool,
}
