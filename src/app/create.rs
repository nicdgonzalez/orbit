use std::path::{Path, PathBuf};
use std::{fs, io};

use thiserror::Error;

use crate::session::{Session, SessionError};
use crate::session_id::{FromDirectoryError, SessionId};

#[derive(Debug, Error)]
pub enum StartSessionError {
    #[error("expected path to point to a directory")]
    NotADirectory,

    #[error("path must not be empty")]
    EmptyPath,

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

    #[error("failed to create session")]
    CreateSession(#[source] SessionError),

    #[error("failed to run setup script")]
    RunSetupScript(#[source] SessionError),
}

#[expect(clippy::missing_panics_doc)]
pub fn start_session(path: impl AsRef<Path>) -> Result<Session, StartSessionError> {
    let path = path.as_ref();

    if path.is_empty() {
        return Err(StartSessionError::EmptyPath);
    }

    if !path.is_dir() {
        return Err(StartSessionError::NotADirectory);
    }

    let session_id = SessionId::from_directory(path).map_err(StartSessionError::CreateSessionId)?;
    let session = Session::new(session_id);

    // If not already exists, create a fallback setup script.
    let config_directory = get_config_directory().ok_or(StartSessionError::GetConfigDirectory)?;
    let fallback_script = config_directory.join("orbit.sh");

    if !fallback_script
        .try_exists()
        .map_err(StartSessionError::CheckFallbackScript)?
    {
        let contents = include_str!("../../orbit.default.sh");
        fs::write(&fallback_script, contents).map_err(StartSessionError::SaveFallbackScript)?;
    }

    session
        .create(path)
        .map_err(StartSessionError::CreateSession)?;

    let custom_script = path.join("orbit.sh");
    let has_custom_script = custom_script
        .try_exists()
        .map_err(StartSessionError::CheckCustomScript)?;

    let script = if has_custom_script {
        custom_script
    } else {
        fallback_script
    };

    let script_path = script
        .to_str()
        .expect("path already validated when creating session ID");
    let command = format!("bash \"{script_path}\"");

    session
        .send_keys(&command)
        .map_err(StartSessionError::RunSetupScript)?;

    Ok(session)
}

fn get_config_directory() -> Option<PathBuf> {
    dirs::config_local_dir().map(|mut path| {
        path.push("orbit");
        path
    })
}
