use clap::Args;

use super::{IdentifierTypeArg, InputArgs};

#[derive(Debug, Args)]
#[command(next_help_heading = "Validation options")]
pub struct ValidateArgs {
    #[command(flatten)]
    pub input: InputArgs,

    /// Validate one identifier family instead of UUID only.
    #[arg(
        short = 't',
        long = "type",
        value_enum,
        value_name = "TYPE",
        help_heading = "Validation options"
    )]
    pub kind: Option<IdentifierTypeArg>,
}
