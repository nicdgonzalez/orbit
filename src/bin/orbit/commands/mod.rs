use clap_verbosity_flag::Verbosity;

mod attach;
mod completions;
mod detach;
mod init;
mod kill;
mod util;

/// Executes the user-selected subcommand.
pub fn run(args: Parser) -> anyhow::Result<()> {
    let ctx = Context {};

    match args.subcommand {
        Subcommand::Completions(handler) => handler.run(ctx),
        Subcommand::Init(handler) => handler.run(ctx),
        Subcommand::Attach(handler) => handler.run(ctx),
        Subcommand::Detach(handler) => handler.run(ctx),
        Subcommand::Kill(handler) => handler.run(ctx),
        Subcommand::Util(handler) => handler.run(ctx),
    }
}

/// Represents a subcommand that can be executed.
pub(super) trait Run {
    /// Executes the subcommand.
    fn run(self, ctx: Context) -> anyhow::Result<()>;
}

/// Shared state for command execution.
pub(super) struct Context {}

/// Command-line interface for the application.
#[derive(clap::Parser)]
#[clap(about = "🚀 Quickly spin up pre-configured tmux sessions")]
pub struct Parser {
    #[clap(subcommand)]
    pub subcommand: Subcommand,

    #[clap(flatten)]
    pub verbosity: Verbosity,
}

/// Commands supported by the application.
#[derive(clap::Subcommand)]
pub enum Subcommand {
    /// Generates an auto-complete script for the specified shell.
    #[clap(hide = true)]
    Completions(completions::Args),

    /// Generates a new setup script in the current directory.
    Init(init::InitArgs),

    /// Opens the target session.
    Attach(attach::Args),

    /// Closes the current session.
    Detach(detach::Args),

    /// Kills the target session.
    Kill(kill::Args),

    /// Utility helpers for setup scripts.
    #[clap(subcommand)]
    Util(util::Subcommand),
}
