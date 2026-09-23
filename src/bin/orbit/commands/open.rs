use std::path::PathBuf;

use anyhow::Context as _;
use orbit::app::open_session;

use crate::commands::{Context, Run};

#[derive(clap::Args)]
pub struct Args {
    query: String,
}

impl Run for Args {
    fn run(self, _: Context) -> anyhow::Result<()> {
        // TODO: Use fzf to get the path to pass to `open_session`.

        _ = open_session(PathBuf::from(self.query)).context("failed to open session")?;

        Ok(())
    }
}
