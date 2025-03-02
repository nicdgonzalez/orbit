mod attach;
mod detach;
mod init;

#[derive(clap::Subcommand)]
pub enum Command {
    /// Initialize a new orbit project in the current directory.
    Init,
    /// Open a tmux session.
    Attach(attach::Args),
    /// Close the current tmux session.
    Detach,
}

pub fn handle_command(command: &Command) -> Result<(), Box<dyn std::error::Error>> {
    match &command {
        Command::Init => Ok(init::run()?),
        Command::Attach(ref args) => Ok(attach::run(args)?),
        Command::Detach => Ok(detach::run()?),
    }
}
