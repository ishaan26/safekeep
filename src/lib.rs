mod error;

pub use error::BackupError;
pub use safekeep_derive::BackupType;
use serde::Serialize;

use std::path::{Path, PathBuf};

/// Options and flags which can be used to configure how data is backed.
pub struct BackupOptions {
    path: PathBuf,
    backup_data: Vec<Box<dyn BackupType>>,
    with_compression: bool,
    with_encryption: bool,
}

/// Only the types that implement this trait can be backed-up.
///
/// Use `#[derive(BackupType)]` with `#[safekeep(format = "json")]` to automatically
/// implement this trait. Supported formats: `json`, `yaml`, `toml`.
///
/// # Example
/// ```ignore
/// use safekeep::BackupType;
/// use serde::Serialize;
///
/// #[derive(BackupType, Serialize)]
/// #[safekeep(format = "json")]
/// struct MyData {
///     name: String,
/// }
/// ```
pub trait BackupType {
    /// Serialize this type to bytes for backup.
    fn backup_bytes(&self) -> Result<Vec<u8>, BackupError>;

    /// Returns the file extension for this backup format.
    fn extension(&self) -> &'static str;

    /// Returns the name of the type.
    fn name(&self) -> &'static str;
}

// --- Implementations for Standard Types (Default to JSON) ---

// Macro for numeric types
macro_rules! impl_backup_for_numeric {
    ($($t:ty),*) => {
        $(
            impl BackupType for $t {
                fn backup_bytes(&self) -> Result<Vec<u8>, BackupError> {
                    #[cfg(feature = "json")]
                    {
                        serde_json::to_vec(self).map_err(|e| BackupError::Serialization(e.to_string()))
                    }
                    #[cfg(not(feature = "json"))]
                    {
                         Err(BackupError::Serialization("JSON feature required for default primitive backup".into()))
                    }
                }

                fn extension(&self) -> &'static str {
                    "json"
                }

                fn name(&self) -> &'static str {
                    stringify!($t)
                }
            }
        )*
    };
}

impl_backup_for_numeric!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64, bool
);

impl BackupType for String {
    fn backup_bytes(&self) -> Result<Vec<u8>, BackupError> {
        #[cfg(feature = "json")]
        {
            serde_json::to_vec(self).map_err(|e| BackupError::Serialization(e.to_string()))
        }
        #[cfg(not(feature = "json"))]
        {
            Err(BackupError::Serialization(
                "JSON feature required for default String backup".into(),
            ))
        }
    }

    fn extension(&self) -> &'static str {
        "json"
    }

    fn name(&self) -> &'static str {
        "String"
    }
}

impl BackupType for &str {
    fn backup_bytes(&self) -> Result<Vec<u8>, BackupError> {
        #[cfg(feature = "json")]
        {
            serde_json::to_vec(self).map_err(|e| BackupError::Serialization(e.to_string()))
        }
        #[cfg(not(feature = "json"))]
        {
            Err(BackupError::Serialization(
                "JSON feature required for default &str backup".into(),
            ))
        }
    }

    fn extension(&self) -> &'static str {
        "json"
    }

    fn name(&self) -> &'static str {
        "str"
    }
}

impl<T> BackupType for Vec<T>
where
    T: Serialize + 'static,
{
    fn backup_bytes(&self) -> Result<Vec<u8>, BackupError> {
        #[cfg(feature = "json")]
        {
            serde_json::to_vec(self).map_err(|e| BackupError::Serialization(e.to_string()))
        }
        #[cfg(not(feature = "json"))]
        {
            Err(BackupError::Serialization(
                "JSON feature required for default Vec backup".into(),
            ))
        }
    }

    fn extension(&self) -> &'static str {
        "json"
    }

    fn name(&self) -> &'static str {
        "Vec"
    }
}

impl BackupOptions {
    /// Creates a backup engine with the following defaults:
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
    use serde::Serialize;

    #[derive(BackupType, Serialize)]
    #[safekeep(format = "json")]
    struct MockBackup {
        name: String,
    }

    #[derive(BackupType, Serialize)]
    #[safekeep(format = "json")]
    struct MockBackupOther(bool);

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
        opts.backup_data(MockBackup {
            name: "test".to_string(),
        });
        assert_eq!(opts.backup_data.len(), 1);
        opts.backup_data(MockBackupOther(true));
        assert_eq!(opts.backup_data.len(), 2);
        opts.backup_data(Vec::from(["s", "t"]));
        assert_eq!(opts.backup_data.len(), 3);
        opts.backup_data("Just a &str");
        assert_eq!(opts.backup_data.len(), 4);
        opts.backup_data(1);
        assert_eq!(opts.backup_data.len(), 5);
    }

    #[test]
    fn test_flags_configuration() {
        let mut opts = BackupOptions::new();

        opts.with_compression(false);
        assert!(!opts.with_compression);

        opts.with_encryption(false);
        assert!(!opts.with_encryption);
    }

    #[test]
    fn test_backup_serialization() {
        let data = MockBackup {
            name: "test".to_string(),
        };
        let result = data.backup_bytes().unwrap();
        assert_eq!(data.extension(), "json");
        assert_eq!(data.name(), "MockBackup");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_std_types_backup() {
        let vec_data = vec![1, 2, 3];
        let vec_result = vec_data.backup_bytes().unwrap();
        assert_eq!(vec_data.extension(), "json");
        assert_eq!(vec_data.name(), "Vec");
        assert!(!vec_result.is_empty());

        let str_data = "hello".to_string();
        let str_result = str_data.backup_bytes().unwrap();
        assert_eq!(str_data.extension(), "json");
        assert_eq!(str_data.name(), "String");
        assert!(!str_result.is_empty());

        let num_data = 42u32;
        let num_result = num_data.backup_bytes().unwrap();
        assert_eq!(num_data.extension(), "json");
        assert_eq!(num_data.name(), "u32");
        assert!(!num_result.is_empty());
    }
}
