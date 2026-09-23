use crate::commands::Run;

#[derive(clap::Args)]
pub struct InitArgs;

impl Run for InitArgs {
    fn run(self, _: super::Context) -> anyhow::Result<()> {
        todo!()
    }
}
