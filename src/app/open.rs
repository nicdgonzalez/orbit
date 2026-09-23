use std::path::{self, Path, PathBuf};
use std::{fs, io};

use path_clean::PathClean as _;
use thiserror::Error;

use crate::session::{Session, SessionError, in_tmux_session};
use crate::session_id::SessionId;

/// Errors that can occur while opening a session.
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

    #[error("failed to get user's config directory")]
    GetConfigDirectory,

    #[error("failed to check if fallback script already exists")]
    CheckFallbackScript(#[source] io::Error),

    #[error("failed to save fallback script")]
    SaveFallbackScript(#[source] io::Error),

    #[error("failed to check if custom script already exists")]
    CheckCustomScript(#[source] io::Error),

    #[error("failed to check if session already exists")]
    CheckSessionExists(#[source] SessionError),

    #[error("failed to create session")]
    CreateSession(#[source] SessionError),

    #[error("failed to run setup script")]
    RunSetupScript(#[source] SessionError),

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
/// - Failed to create target session
/// - Failed to attach client to target session
#[expect(clippy::missing_panics_doc)]
pub fn open_session(path: impl AsRef<Path>) -> Result<Session, OpenSessionError> {
    let path = path.as_ref();

    if path.is_empty() {
        return Err(OpenSessionError::EmptyPath);
    }

    if !path.is_dir() {
        return Err(OpenSessionError::NotADirectory);
    }

    let session_id = session_id_from_directory(path).map_err(OpenSessionError::CreateSessionId)?;
    let session = Session::new(session_id);

    // If not already exists, create a fallback setup script.
    let config_directory = get_config_directory().ok_or(OpenSessionError::GetConfigDirectory)?;
    let fallback_script = config_directory.join("orbit.sh");

    if !fallback_script
        .try_exists()
        .map_err(OpenSessionError::CheckFallbackScript)?
    {
        let contents = include_str!("../../orbit.default.sh");
        fs::write(&fallback_script, contents).map_err(OpenSessionError::SaveFallbackScript)?;
    }

    // If not already exists, create the session and run the setup script.
    let session_exists = session
        .exists()
        .map_err(OpenSessionError::CheckSessionExists)?;

    if !session_exists {
        session
            .create(path)
            .map_err(OpenSessionError::CreateSession)?;

        let custom_script = path.join("orbit.sh");
        let has_custom_script = custom_script
            .try_exists()
            .map_err(OpenSessionError::CheckCustomScript)?;

        let script = if has_custom_script {
            custom_script
        } else {
            fallback_script
        };

        let script_path = script.to_str().expect("path has already been validated");
        let command = format!("bash \"{script_path}\"");

        session
            .send_keys(&command)
            .map_err(OpenSessionError::RunSetupScript)?;
    }

    if in_tmux_session() {
        session.switch().map_err(OpenSessionError::SwitchSession)?;
    } else {
        session.attach().map_err(OpenSessionError::AttachSession)?;
    }

    Ok(session)
}

/// Errors that can occur when creating a [`SessionId`] from a directory name.
#[derive(Debug, Error)]
pub enum FromDirectoryError {
    #[error("path must not be empty")]
    EmptyPath,

    #[error("failed to get current directory")]
    GetCurrentDirectory(#[source] io::Error),

    #[error("path contains invalid Unicode")]
    InvalidUnicode,
}

/// Creates a new [`SessionId`] from a directory name.
fn session_id_from_directory(path: impl AsRef<Path>) -> Result<SessionId, FromDirectoryError> {
    let path = path.as_ref();

    if path.is_empty() {
        return Err(FromDirectoryError::EmptyPath);
    }

    let path = path.clean();
    let path = path::absolute(&path).map_err(FromDirectoryError::GetCurrentDirectory)?;

    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or(FromDirectoryError::InvalidUnicode)?;

    Ok(SessionId::normalize(name))
}

fn get_config_directory() -> Option<PathBuf> {
    dirs::config_local_dir().map(|mut path| {
        path.push("orbit");
        path
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn from_empty_path() {
        let path = PathBuf::from("");
        let error = session_id_from_directory(&path).unwrap_err();
        assert!(matches!(error, FromDirectoryError::EmptyPath));
    }

    #[test]
    fn from_path_with_trailing_separator() {
        let path = PathBuf::from("./orbit/");
        let session_id = session_id_from_directory(&path).unwrap();
        assert_eq!(session_id.as_str(), "orbit");
    }

    #[test]
    fn from_path_with_special_parent_component() {
        let path = PathBuf::from("./foo/bar/..");
        let session_id = session_id_from_directory(&path).unwrap();
        assert_eq!(session_id.as_str(), "foo");
    }
}
