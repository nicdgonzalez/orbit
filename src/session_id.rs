use std::io;
use std::path::{self, Path};

use path_clean::PathClean as _;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionId(String);

impl SessionId {
    /// Creates a new [`SessionId`] that is known to be valid.
    ///
    /// # Errors
    ///
    /// Returns an error if `id` is an empty string or contains a colon, period,
    /// or any non-alphanumeric characters.
    ///
    /// # Examples
    ///
    /// ```
    /// # use orbit::session_id::SessionId;
    /// assert!(SessionId::new("orbit".to_owned()).is_ok());
    /// assert!(SessionId::new("オルビート".to_owned()).is_ok());
    /// assert!(SessionId::new("".to_owned()).is_err());
    /// assert!(SessionId::new("orbit.rs".to_owned()).is_err());
    /// assert!(SessionId::new("orbit:)".to_owned()).is_err());
    /// ```
    pub fn new(id: String) -> Result<Self, InvalidSessionId> {
        if id.is_empty() {
            return Err(InvalidSessionId::Empty);
        }

        if !id.chars().all(Self::is_valid_character) {
            return Err(InvalidSessionId::InvalidCharacters);
        }

        Ok(Self(id))
    }

    /// Creates a new [`SessionId`] from a directory name.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - Provided `path` is empty
    /// - Failed to get current directory
    /// - Directory name contains invalid Unicode
    pub fn from_directory(path: impl AsRef<Path>) -> Result<Self, FromDirectoryError> {
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

        Ok(Self::normalize(name))
    }

    pub(crate) fn from_tmux(id: String) -> Self {
        Self(id)
    }

    /// Creates a new [`SessionId`] with invalid characters replaced by an underscore.
    ///
    /// # Examples
    ///
    /// ```
    /// use orbit::session_id::SessionId;
    /// assert_eq!(SessionId::normalize("orbit.rs").as_str(), "orbit_rs");
    /// assert_eq!(SessionId::normalize("orbit:)").as_str(), "orbit__");
    /// assert_eq!(SessionId::normalize("hello orbit").as_str(), "hello_orbit");
    /// ```
    #[must_use]
    pub fn normalize(id: &str) -> Self {
        let normalized = id
            .chars()
            .map(|c| if Self::is_valid_character(c) { c } else { '_' })
            .collect();

        Self(normalized)
    }

    fn is_valid_character(c: char) -> bool {
        c.is_alphanumeric() || c == '-' || c == '_'
    }

    /// Returns the inner value as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Error)]
pub enum InvalidSessionId {
    #[error("session id must not be empty")]
    Empty,

    #[error("session id contains invalid characters")]
    InvalidCharacters,
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn from_empty_path() {
        let path = PathBuf::from("");
        let error = SessionId::from_directory(&path).unwrap_err();
        assert!(matches!(error, FromDirectoryError::EmptyPath));
    }

    #[test]
    fn from_path_with_trailing_separator() {
        let path = PathBuf::from("./orbit/");
        let session_id = SessionId::from_directory(&path).unwrap();
        assert_eq!(session_id.as_str(), "orbit");
    }

    #[test]
    fn from_path_with_special_parent_component() {
        let path = PathBuf::from("./foo/bar/..");
        let session_id = SessionId::from_directory(&path).unwrap();
        assert_eq!(session_id.as_str(), "foo");
    }
}
