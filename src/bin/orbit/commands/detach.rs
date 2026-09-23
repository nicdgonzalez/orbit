use anyhow::{Context as _, bail};
use orbit::session::{Session, in_tmux_session};

use crate::commands::{Context, Run};

#[derive(clap::Args)]
pub struct Args;

impl Run for Args {
    fn run(self, _: Context) -> anyhow::Result<()> {
        if !in_tmux_session() {
            bail!("not in a tmux session");
        }

        Session::detach().context("failed to detach from session")?;

        Ok(())
    }
}
