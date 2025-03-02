//! Implementation for the `stop` command.

use std::{env, process};

pub fn run() -> Result<(), DetachError> {
    if !env::var("TMUX").is_ok() {
        return Err(DetachError::NotInsideTmux);
    }

    let status = process::Command::new("tmux")
        .arg("detach-client")
        .status()
        .expect("failed to execute tmux command");

    assert!(status.success());

    Ok(())
}

/// There was a problem detaching the client from a tmux session.
#[derive(Debug)]
pub enum DetachError {
    NotInsideTmux,
}

impl std::error::Error for DetachError {}

impl std::fmt::Display for DetachError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInsideTmux => write!(f, "not inside a tmux session"),
        }
    }
}
