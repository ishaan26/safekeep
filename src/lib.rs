mod error;

use error::BackupError;
use std::path::{Path, PathBuf};

/// Options and flags which can be used to configure how data is backed.
pub struct BackupOptions {
    path: PathBuf,
    backup_data: Vec<Box<dyn BackupType>>,
    with_compression: bool,
    with_encryption: bool,
}

/// Only the types that implement this trait can be backed-up.
pub trait BackupType {
    fn backup(&self) -> Result<(), BackupError>;
}

impl<T> BackupType for Vec<T> {
    /// TODO: What to do inside here?
    fn backup(&self) -> Result<(), BackupError> {
        Ok(())
    }
}

impl BackupOptions {
    /// Creats a backup engine with the following defaults:
    /// - Path: Current directory
    /// - compression: yes
    /// - encryption: yes
    pub fn new() -> Self {
        Self {
            path: PathBuf::from("./"),
            backup_data: Vec::new(),
            with_compression: true,
            with_encryption: true,
        }
    }

    /// Sets the path where the backup will be created
    pub fn path(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.path = path.as_ref().to_path_buf();
        self
    }

    /// Sets the data to be backed up. Run it multiple times to add more types
    pub fn backup_data(&mut self, data: impl BackupType + 'static) -> &mut Self {
        self.backup_data.push(Box::new(data));
        self
    }

    /// To compress or not to compress
    pub fn with_compression(&mut self, compression: bool) -> &mut Self {
        self.with_compression = compression;
        self
    }

    /// To encrypt or not to encrypt
    pub fn with_encryption(&mut self, encryption: bool) -> &mut Self {
        self.with_encryption = encryption;
        self
    }

    /// Run the backup
    pub async fn run(self) -> Result<(), BackupError> {
        todo!()
    }
}

impl Default for BackupOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockBackup;
    #[allow(dead_code)]
    struct MockBackupOther(bool);

    impl BackupType for MockBackup {
        fn backup(&self) -> Result<(), BackupError> {
            Ok(())
        }
    }

    impl BackupType for MockBackupOther {
        fn backup(&self) -> Result<(), BackupError> {
            Ok(())
        }
    }

    #[test]
    fn test_defaults() {
        let opts = BackupOptions::new();
        assert_eq!(opts.path, PathBuf::from("./"));
        assert!(opts.backup_data.is_empty());
        assert!(opts.with_compression);
        assert!(opts.with_encryption);
    }

    #[test]
    fn test_path_configuration() {
        let mut opts = BackupOptions::new();
        opts.path("/tmp/backup");
        assert_eq!(opts.path, PathBuf::from("/tmp/backup"));
    }

    #[test]
    fn test_backup_data_configuration() {
        let mut opts = BackupOptions::new();
        opts.backup_data(MockBackup);
        assert_eq!(opts.backup_data.len(), 1);
        opts.backup_data(MockBackupOther(true));
        assert_eq!(opts.backup_data.len(), 2);
        opts.backup_data(Vec::from(["s", "t"]));
        assert_eq!(opts.backup_data.len(), 3);
    }

    #[test]
    fn test_flags_configuration() {
        let mut opts = BackupOptions::new();

        opts.with_compression(false);
        assert!(!opts.with_compression);

        opts.with_encryption(false);
        assert!(!opts.with_encryption);
    }
}
