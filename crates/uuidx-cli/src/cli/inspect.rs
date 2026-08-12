use clap::Args;

use super::InputArgs;

#[derive(Debug, Args)]
#[command(next_help_heading = "Inspection options")]
pub struct InspectArgs {
    #[command(flatten)]
    pub input: InputArgs,

    /// Show decoded fields and bit-layout details in pretty output.
    #[arg(short = 'L', long, help_heading = "Inspection options")]
    pub layout: bool,

    /// Omit time-based UUID node IDs while keeping their classification.
    #[arg(short = 'r', long, help_heading = "Inspection options")]
    pub redact_sensitive: bool,
}
