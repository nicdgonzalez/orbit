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
