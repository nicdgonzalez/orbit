mod commands;

use std::process;

use clap::Parser;
use colored::Colorize;

/// 🚀 A session manager for tmux.
#[derive(clap::Parser)]
#[command(
    version,
    after_help = "Repository: https://github.com/nicdgonzalez/orbit"
)]
pub struct Args {
    #[command(subcommand)]
    pub command: commands::Command,
}

fn main() {
    if let Err(why) = try_main() {
        eprintln!("{}: {}", "error".bold().red(), why);
        process::exit(1);
    }
}

fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    Ok(commands::handle_command(&args.command)?)
}
