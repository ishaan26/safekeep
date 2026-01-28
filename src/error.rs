use serde::Serialize;
use thiserror::Error;

/// Represents errors that can occur during backup and restore operations.
#[derive(Debug, Error, Serialize)]
pub enum BackupError {
    /// The provided backup directory is invalid or does not exist.
    #[error("Invalid backup directory: {0}")]
    InvalidDirectory(String),
    /// The backup index file is corrupted or inconsistent.
    #[error("Backup Index got corrupted: {0}")]
    IndexCorrupted(String),
    /// An I/O error occurred during file operations.
    #[error("IO error: {0}")]
    Io(String),
    /// An error occurred during database backup.
    #[error("Database backup failed: {0}")]
    Database(String),
    /// Serialisation Error
    #[error("Serialization of the provided type failed: {0}")]
    Ser(String),
    /// An internal error occurred (e.g., serialization, Tauri, or other unexpected errors).
    #[error("Internal Error Occured: {0}")]
    Other(String),
}

impl From<std::io::Error> for BackupError {
    fn from(err: std::io::Error) -> Self {
        BackupError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for BackupError {
    fn from(err: serde_json::Error) -> Self {
        BackupError::Ser(err.to_string())
    }
}
