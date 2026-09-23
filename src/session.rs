use std::path::Path;
use std::process::{Command, Stdio};
use std::string::FromUtf8Error;
use std::{env, io};

use thiserror::Error;

use crate::session_id::SessionId;

/// Returns `true` if the client is attached to *any* sessions.
///
/// This function checks the `TMUX` environment variable to decide if the client is connected
/// to a session.
#[must_use]
pub fn in_tmux_session() -> bool {
    env::var_os("TMUX").is_some()
}

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
    #[error("not in a tmux session")]
    NotInTmuxSession,

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

    /// Starts a new session at the provided `path`.
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

    /// Destroys the session.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - Failed to execute tmux in a child process
    /// - Child process returns an unexpected exit code
    pub fn kill(&self) -> Result<(), SessionError> {
        // The leading equal sign tells tmux to check for an exact match.
        let id_exact_match = format!("={}", self.id.as_str());

        let status = Command::new("tmux")
            .args(["kill-session", "-t", &id_exact_match])
            .status()
            .map_err(SessionError::SpawnProcess)?;

        match status.code() {
            Some(0) => Ok(()),
            Some(code) => Err(SessionError::UnexpectedExitCode(code)),
            None => Err(SessionError::TerminatedBySignal),
        }
    }

    /// Destroys the current session.
    ///
    /// A more destructive version of [`Self::kill`].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - Client is not currently in a tmux session
    /// - Failed to execute tmux in a child process
    /// - Child process returns an unexpected exit code
    pub fn kill_current() -> Result<(), SessionError> {
        if !in_tmux_session() {
            return Err(SessionError::NotInTmuxSession);
        }

        let status = Command::new("tmux")
            .arg("kill-session")
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

    /// Detaches the client from the session.
    ///
    /// This function must be called from inside of a tmux session.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - Failed to execute tmux in a child process
    /// - Child process returns an unexpected exit code
    pub fn detach() -> Result<(), SessionError> {
        let status = Command::new("tmux")
            .arg("detach")
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

/// Errors that can occur while getting active tmux sessions.
#[derive(Debug, Error)]
pub enum ListSessionsError {
    #[error("failed to execute tmux in child process")]
    SpawnProcess(#[source] io::Error),

    #[error("process terminated with a non-zero exit code: {0}")]
    UnexpectedExitCode(i32),

    #[error("process terminated by a signal")]
    TerminatedBySignal,

    #[error("output is not valid UTF-8")]
    InvalidOutput(#[source] FromUtf8Error),
}

/// Returns a list of active session names.
///
/// # Errors
///
/// Returns an error if:
///
/// - Failed to execute tmux in a child process
/// - Child process returns an unexpected exit code
/// - Child process terminated by a signal
/// - tmux returns invalid output
pub fn list_sessions() -> Result<Vec<SessionId>, ListSessionsError> {
    let output = Command::new("tmux")
        .args(["list-sessions", "-F", "#{session_name}"])
        .output()
        .map_err(ListSessionsError::SpawnProcess)?;

    match output.status.code() {
        Some(0) => {} // Process terminated without any errors; no early return.
        Some(code) => return Err(ListSessionsError::UnexpectedExitCode(code)),
        None => return Err(ListSessionsError::TerminatedBySignal),
    }

    let stdout = String::from_utf8(output.stdout).map_err(ListSessionsError::InvalidOutput)?;
    let session_names = stdout
        .lines()
        .map(|line| SessionId::from_tmux(line.to_owned()))
        .collect::<Vec<_>>();

    Ok(session_names)
}
