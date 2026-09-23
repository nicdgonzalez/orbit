use clap_verbosity_flag::Verbosity;

mod completions;
mod open;

/// Executes the user-selected subcommand.
pub fn run(args: Parser) -> anyhow::Result<()> {
    let ctx = Context {};

    match args.subcommand {
        Subcommand::Completions(handler) => handler.run(ctx),
        Subcommand::Open(handler) => handler.run(ctx),
    }
}

/// Represents a subcommand that can be executed.
pub(super) trait Run
where
    Self: clap::Args,
{
    /// Executes the subcommand.
    fn run(self, ctx: Context) -> anyhow::Result<()>;
}

/// Shared state for command execution.
pub(super) struct Context {}

/// Command-line interface for the application.
#[derive(clap::Parser)]
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

    /// Attaches the client to a session.
    Open(open::Args),
}
