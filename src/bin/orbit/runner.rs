use clap::Parser as _;

use crate::commands::{self, Parser};
use crate::error::{ExitCode, report};

/// Executes the application.
///
/// If the application completes successfully, [`ExitCode::Success`] is returned.
/// Otherwise, the error is reported and converted into the appropriate exit code.
#[must_use]
pub fn run() -> ExitCode {
    match try_run() {
        Ok(()) => ExitCode::Success,
        Err(err) => report(err.as_ref()),
    }
}

/// Executes the application, propagating errors to the caller.
fn try_run() -> anyhow::Result<()> {
    let args = Parser::parse();
    commands::run(args)
}
