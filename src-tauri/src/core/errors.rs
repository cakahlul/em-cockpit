//! Application error types
//!
//! Centralized error handling following the Error Hierarchy pattern.

use thiserror::Error;

/// Top-level application error
#[derive(Error, Debug)]
pub enum CockpitError {
    #[error("Security error: {0}")]
    Security(#[from] crate::security::CredentialError),

    #[error("Cache error: {0}")]
    Cache(#[from] crate::services::CacheError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Integration error: {0}")]
    Integration(String),

    #[error("Service error: {0}")]
    Service(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<serde_json::Error> for CockpitError {
    fn from(err: serde_json::Error) -> Self {
        CockpitError::Serialization(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = CockpitError::Config("Invalid setting".to_string());
        assert_eq!(format!("{}", err), "Configuration error: Invalid setting");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let cockpit_err: CockpitError = io_err.into();
        
        assert!(matches!(cockpit_err, CockpitError::Io(_)));
    }
}
