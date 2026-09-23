use std::io;
use std::io::Write as _;
use std::path::PathBuf;

use anyhow::Context as _;
use orbit::session_id::SessionId;

use crate::commands::{Context, Run};

#[derive(Debug, Clone, clap::Args)]
pub struct Args {
    path: PathBuf,
}

impl Run for Args {
    fn run(self, _: Context) -> anyhow::Result<()> {
        let session_id =
            SessionId::from_directory(&self.path).context("failed to create session ID")?;

        writeln!(io::stdout(), "{}", session_id.as_str()).ok();

        Ok(())
    }
}
