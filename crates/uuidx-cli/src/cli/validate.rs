use clap::Args;

use super::InputArgs;

#[derive(Debug, Args)]
#[command(next_help_heading = "Validation options")]
pub struct ValidateArgs {
    #[command(flatten)]
    pub input: InputArgs,
}
