mod error;

use error::BackupError;
use std::path::Path;

/// Only the types that implement this trait can be backed-up.
pub trait BackupType {
    fn backup(&self, path: &Path) -> Result<(), BackupError>;
}
