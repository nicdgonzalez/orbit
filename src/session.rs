use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

use thiserror::Error;

use crate::session_id::SessionId;

/// Simple wrapper for operating on a tmux session.
///
/// Note that only the necessary subset of features and operations are implemented.
#[derive(Debug, Clone)]
pub struct Session {
    id: SessionId,
}

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

impl Session {
    /// Creates a new [`Session`] with the provided `id`.
    #[must_use]
    pub const fn new(id: SessionId) -> Self {
        Self { id }
    }

    /// Checks if a session currently exists.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - Failed to execute tmux in a child process
    /// - Child process returns an unexpected exit code
    pub fn exists(&self) -> Result<bool, SessionError> {
        // The leading equal sign tells tmux to check for an exact match.
        let id_exact_match = format!("={}", self.id.as_str());

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

    /// Creates a new session at the provided `path`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - Provided `path` contains invalid Unicode
    /// - Failed to execute tmux in a child process
    /// - Child process returns an unexpected exit code
    pub fn create(&self, path: impl AsRef<Path>) -> Result<(), SessionError> {
        let directory = path.as_ref().to_str().ok_or(SessionError::InvalidUnicode)?;

        let status = Command::new("tmux")
            .args(["new-session", "-d", "-s", self.id.as_str(), "-c", directory])
            .status()
            .map_err(SessionError::SpawnProcess)?;

        match status.code() {
            Some(0) => Ok(()),
            Some(code) => Err(SessionError::UnexpectedExitCode(code)),
            None => Err(SessionError::TerminatedBySignal),
        }
    }

    /// Connects the client to this session.
    ///
    /// This function must be called from outside of a tmux session to avoid nesting.
    /// Use [`Self::switch`] instead if the client is already in a session and needs to be moved.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - Failed to execute tmux in a child process
    /// - Child process returns an unexpected exit code
    pub fn attach(&self) -> Result<(), SessionError> {
        // The leading equal sign tells tmux to check for an exact match.
        let id_exact_match = format!("={}", self.id.as_str());

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

    /// Moves the client from another session into this session.
    ///
    /// This function must be called from inside a tmux session. Otherwise, use [`Self::attach`].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - Failed to execute tmux in a child process
    /// - Child process returns an unexpected exit code
    pub fn switch(&self) -> Result<(), SessionError> {
        // The leading equal sign tells tmux to check for an exact match.
        let id_exact_match = format!("={}", self.id.as_str());

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

    /// Sends a command to the session.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - Failed to execute tmux in a child process
    /// - Child process returns an unexpected exit code
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use orbit::session::Session;
    /// # use orbit::session_id::SessionId;
    /// let id = SessionId::new("foo");
    /// let session = Session::new(id);
    /// assert!(session.exists().unwrap_or(false));
    /// session.send_keys("echo 'Hello, Orbit!'")?;
    /// ```
    pub fn send_keys(&self, command: &str) -> Result<(), SessionError> {
        let status = Command::new("tmux")
            .args(["send-keys", "-t", self.id.as_str(), command, "Enter"])
            .status()
            .map_err(SessionError::SpawnProcess)?;

        match status.code() {
            Some(0) => Ok(()),
            Some(code) => Err(SessionError::UnexpectedExitCode(code)),
            None => Err(SessionError::TerminatedBySignal),
        }
    }
}
