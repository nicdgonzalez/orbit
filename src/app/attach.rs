use std::path::Path;

use thiserror::Error;

use crate::app::{StartSessionError, start_session};
use crate::session::{Session, SessionError, in_tmux_session};
use crate::session_id::{FromDirectoryError, SessionId};

/// Errors that can occur while starting a session.
#[derive(Debug, Error)]
pub enum OpenSessionError {
    #[error("expected path to point to a directory")]
    NotADirectory,

    #[error("path must not be empty")]
    EmptyPath,

    #[error("path contains invalid Unicode")]
    InvalidUnicode,

    #[error("failed to create session ID from directory name")]
    CreateSessionId(#[source] FromDirectoryError),

    #[error("failed to check if session already exists")]
    CheckSessionExists(#[source] SessionError),

    #[error("failed to start session")]
    StartSession(#[source] StartSessionError),

    #[error("failed to switch to target session")]
    SwitchSession(#[source] SessionError),

    #[error("failed to attach to target session")]
    AttachSession(#[source] SessionError),
}

/// Opens the session at `path`, creating it if it doesn't already exist.
///
/// # Errors
///
/// Returns an error if:
///
/// - Provided `path` is empty
/// - Provided `path` does not point to a directory
/// - Failed to create [`SessionId`] from the provided directory
/// - Failed to check if target session already exists
/// - Failed to start target session
/// - Failed to attach client to target session
pub fn attach_to_session(path: impl AsRef<Path>) -> Result<Session, OpenSessionError> {
    let path = path.as_ref();

    if path.is_empty() {
        return Err(OpenSessionError::EmptyPath);
    }

    if !path.is_dir() {
        return Err(OpenSessionError::NotADirectory);
    }

    let session_id = SessionId::from_directory(path).map_err(OpenSessionError::CreateSessionId)?;
    let session = Session::new(session_id);

    let session_exists = session
        .exists()
        .map_err(OpenSessionError::CheckSessionExists)?;

    if !session_exists {
        start_session(path).map_err(OpenSessionError::StartSession)?;
    }

    if in_tmux_session() {
        session.switch().map_err(OpenSessionError::SwitchSession)?;
    } else {
        session.attach().map_err(OpenSessionError::AttachSession)?;
    }

    Ok(session)
}
