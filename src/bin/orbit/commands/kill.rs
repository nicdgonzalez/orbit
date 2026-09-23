use std::path::PathBuf;

use anyhow::Context as _;
use orbit::session::Session;
use orbit::session_id::SessionId;

use crate::commands::Run;

#[derive(clap::Args)]
pub struct Args {
    #[clap(long)]
    path: Option<PathBuf>,
}

impl Run for Args {
    fn run(self, _: super::Context) -> anyhow::Result<()> {
        if let Some(ref path) = self.path {
            let id = SessionId::from_directory(path).context("failed to create session ID")?;
            let session = Session::new(id);
            session.kill().context("failed to kill session")?;
            return Ok(());
        }

        Session::kill_current().context("failed to kill current session")?;

        Ok(())
    }
}
