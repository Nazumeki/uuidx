mod app;
mod cli;
mod commands;
mod errors;
mod input;
mod output;

use clap::Parser;

fn main() {
    let result = app::run(cli::Cli::parse());
    std::process::exit(result.exit_code());
}
