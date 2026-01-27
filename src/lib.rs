mod error;

use error::BackupError;
use std::path::{Path, PathBuf};

pub struct BackupOptions {
    path: PathBuf,
    backup_data: Vec<Box<dyn BackupType>>,
    no_index: bool,
    with_compression: bool,
    with_encryption: bool,
}

/// Only the types that implement this trait can be backed-up.
pub trait BackupType {
    fn backup(&self, path: &Path) -> Result<(), BackupError>;
}
