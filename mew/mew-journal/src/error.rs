//! Journal error types.

use thiserror::Error;

/// Journal errors.
#[derive(Debug, Error)]
pub enum JournalError {
    /// IO error during journal operations.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid entry format.
    #[error("invalid entry format: {message}")]
    InvalidFormat { message: String },

    /// Entry not found.
    #[error("entry not found at LSN {lsn}")]
    EntryNotFound { lsn: u64 },

    /// Recovery error.
    #[error("recovery error: {message}")]
    RecoveryError { message: String },

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(String),
}

impl JournalError {
    pub fn invalid_format(message: impl Into<String>) -> Self {
        Self::InvalidFormat {
            message: message.into(),
        }
    }

    pub fn entry_not_found(lsn: u64) -> Self {
        Self::EntryNotFound { lsn }
    }

    pub fn recovery_error(message: impl Into<String>) -> Self {
        Self::RecoveryError {
            message: message.into(),
        }
    }

    pub fn serialization(message: impl Into<String>) -> Self {
        Self::Serialization(message.into())
    }
}

/// Result type for journal operations.
pub type JournalResult<T> = Result<T, JournalError>;
