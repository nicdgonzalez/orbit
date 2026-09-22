use std::io;

use clap::CommandFactory as _;
use clap_complete::Shell;

use crate::commands::{Context, Parser, Run};

#[derive(Debug, Clone, clap::Args)]
pub struct Args {
    shell: Shell,
}

impl Run for Args {
    fn run(self, _: Context) -> anyhow::Result<()> {
        let mut command = Parser::command();

        clap_complete::generate(
            self.shell,
            &mut command,
            env!("CARGO_BIN_NAME"),
            &mut io::stdout(),
        );

        Ok(())
    }
}
