use crate::commands::Run;

mod session_id;

#[derive(clap::Subcommand)]
pub enum Subcommand {
    /// Displays the session ID for the given directory.
    SessionId(session_id::Args),
}

impl Run for Subcommand {
    fn run(self, ctx: super::Context) -> anyhow::Result<()> {
        match self {
            Self::SessionId(handler) => handler.run(ctx),
        }
    }
}
