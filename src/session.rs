use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

use thiserror::Error;

use crate::session_id::SessionId;

/// Errors that can occur while creating a session.
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("path contains invalid Unicode")]
    InvalidUnicode,

    #[error("failed to execute tmux in child process")]
    SpawnProcess(#[source] io::Error),

    #[error("process terminated with a non-zero exit code: {0}")]
    UnexpectedExitCode(i32),

    #[error("process terminated by a signal")]
    TerminatedBySignal,
}

/// Checks if a session currently exists.
///
/// # Errors
///
/// Returns an error if:
///
/// - Failed to execute tmux in a child process
/// - Child process returns an unexpected exit code
pub fn session_exists(id: &SessionId) -> Result<bool, SessionError> {
    // The leading equal sign tells tmux to check for an exact match.
    let id_exact_match = format!("={}", id.as_str());

    let status = Command::new("tmux")
        .args(["has-session", "-t", &id_exact_match])
        .stderr(Stdio::null())
        .status()
        .map_err(SessionError::SpawnProcess)?;

    match status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        Some(code) => Err(SessionError::UnexpectedExitCode(code)),
        None => Err(SessionError::TerminatedBySignal),
    }
}

/// Creates a new session named `id` at the provided `path`.
///
/// # Errors
///
/// Returns an error if:
///
/// - Provided `path` contains invalid Unicode
/// - Failed to execute tmux in a child process
/// - Child process returns an unexpected exit code
pub fn create_session(id: &SessionId, path: impl AsRef<Path>) -> Result<(), SessionError> {
    let directory = path.as_ref().to_str().ok_or(SessionError::InvalidUnicode)?;

    let status = Command::new("tmux")
        .args(["new-session", "-d", "-s", id.as_str(), "-c", directory])
        .status()
        .map_err(SessionError::SpawnProcess)?;

    match status.code() {
        Some(0) => Ok(()),
        Some(code) => Err(SessionError::UnexpectedExitCode(code)),
        None => Err(SessionError::TerminatedBySignal),
    }
}

pub fn attach_session(id: &SessionId) -> Result<(), SessionError> {
    // The leading equal sign tells tmux to check for an exact match.
    let id_exact_match = format!("={}", id.as_str());

    let status = Command::new("tmux")
        .args(["attach-session", "-t", &id_exact_match])
        .status()
        .map_err(SessionError::SpawnProcess)?;

    match status.code() {
        Some(0) => Ok(()),
        Some(code) => Err(SessionError::UnexpectedExitCode(code)),
        None => Err(SessionError::TerminatedBySignal),
    }
}

pub fn switch_client(id: &SessionId) -> Result<(), SessionError> {
    // The leading equal sign tells tmux to check for an exact match.
    let id_exact_match = format!("={}", id.as_str());

    let status = Command::new("tmux")
        .args(["switch-client", "-t", &id_exact_match])
        .status()
        .map_err(SessionError::SpawnProcess)?;

    match status.code() {
        Some(0) => Ok(()),
        Some(code) => Err(SessionError::UnexpectedExitCode(code)),
        None => Err(SessionError::TerminatedBySignal),
    }
}

pub fn send_keys(id: &SessionId, command: &str) -> Result<(), SessionError> {
    let status = Command::new("tmux")
        .args(["send-keys", "-t", id.as_str(), command, "Enter"])
        .status()
        .map_err(SessionError::SpawnProcess)?;

    match status.code() {
        Some(0) => Ok(()),
        Some(code) => Err(SessionError::UnexpectedExitCode(code)),
        None => Err(SessionError::TerminatedBySignal),
    }
}
