mod error;

pub use error::BackupError;
pub use safekeep_derive::BackupType;
use serde::Serialize;

use std::{
    fs,
    path::{Path, PathBuf},
};

/// A single backup entry, consisting of the data and a name for the file.
pub struct BackupEntry {
    data: Box<dyn BackupType>,
    name: String,
}

impl<T: NamedBackupType + 'static> From<T> for BackupEntry {
    fn from(data: T) -> Self {
        Self {
            name: data.name().to_string(),
            data: Box::new(data),
        }
    }
}

/// Trait representing a value ready to be backed up with a name.
///
/// Implemented automatically for:
/// - Struct types that `#[derive(BackupType)]` (auto-named by struct name)
/// - Any `BackupType` wrapped with `.with_name("name")`
#[diagnostic::on_unimplemented(
    message = "this type needs an explicit name for backup",
    label = "use `.with_name(\"name\")` to give this value a backup name",
    note = "Types that `#[derive(BackupType)]` are auto-named by their struct name.\nAll other types (Vec, String, primitives, etc.) require `.with_name(\"name\")`."
)]
pub trait BackupInput {
    fn into_entry(self) -> BackupEntry;
}

/// Options and flags which can be used to configure how data is backed.
pub struct BackupOptions {
    path: PathBuf,
    backup_data: Vec<BackupEntry>,
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

/// Marker trait for types that can be backed up using their type name.
///
/// This is automatically implemented by `#[derive(BackupType)]` for structs.
/// Standard types (Vec, String, primitives, etc.) do NOT implement this,
/// requiring explicit naming via `.with_name()`.
pub trait NamedBackupType: BackupType {}

/// A wrapper that provides an explicit name for backup data.
pub struct WithName<T: BackupType> {
    data: T,
    name: String,
}

impl<T: BackupType> BackupType for WithName<T> {
    fn backup_bytes(&self) -> Result<Vec<u8>, BackupError> {
        self.data.backup_bytes()
    }

    fn extension(&self) -> &'static str {
        self.data.extension()
    }

    fn name(&self) -> &'static str {
        // This is a bit of a hack - we return a static str, but our name is owned.
        // In practice, this method won't be called on WithName wrappers
        // because we use Into<BackupEntry> which extracts the name directly.
        self.data.name()
    }
}

/// Extension trait to add explicit naming to any backup type.
pub trait WithNameExt: BackupType + Sized {
    /// Wrap this value with an explicit name for backup.
    fn with_name(self, name: &str) -> WithName<Self>;
}

impl<T: BackupType + Sized> WithNameExt for T {
    fn with_name(self, name: &str) -> WithName<Self> {
        WithName {
            data: self,
            name: name.to_string(),
        }
    }
}

impl<T: NamedBackupType + 'static> BackupInput for T {
    fn into_entry(self) -> BackupEntry {
        self.into()
    }
}

impl<T: BackupType + 'static> BackupInput for WithName<T> {
    fn into_entry(self) -> BackupEntry {
        self.into()
    }
}

impl<T: BackupType + 'static> From<WithName<T>> for BackupEntry {
    fn from(wrapped: WithName<T>) -> Self {
        Self {
            data: Box::new(wrapped.data),
            name: wrapped.name,
        }
    }
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

    /// Add data to be backed up.
    ///
    /// For struct types (deriving `BackupType`), this auto-uses the struct name.
    /// For other types, you must wrap with `.with_name()`:
    ///
    /// ```ignore
    /// .backup(my_struct)                    // OK: uses "MyStruct"
    /// .backup(my_struct.with_name("custom")) // OK: uses "custom"
    /// .backup(vec.with_name("items"))        // OK: uses "items"
    /// .backup(vec)                           // ERROR: type needs an explicit name
    /// ```
    pub fn backup(&mut self, entry: impl BackupInput) -> &mut Self {
        self.backup_data.push(entry.into_entry());
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
    pub fn run(&mut self) -> Result<(), BackupError> {
        for entry in &self.backup_data {
            let bytes = entry.data.backup_bytes()?;
            let ext = entry.data.extension();
            let path = self.path.join(format!("{}.{ext}", entry.name));

            // TODO:
            // - compression
            // - encryption
            // - backup index

            fs::create_dir_all(&self.path)?;
            fs::write(path, bytes)?;
        }

        Ok(())
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
        // Structs work directly (auto-named)
        opts.backup(MockBackup {
            name: "test".to_string(),
        });
        assert_eq!(opts.backup_data.len(), 1);
        assert_eq!(opts.backup_data[0].name, "MockBackup");

        opts.backup(MockBackupOther(true));
        assert_eq!(opts.backup_data.len(), 2);
        assert_eq!(opts.backup_data[1].name, "MockBackupOther");

        // Non-structs require .with_name()
        opts.backup(Vec::from(["s", "t"]).with_name("vec_data"));
        assert_eq!(opts.backup_data.len(), 3);
        assert_eq!(opts.backup_data[2].name, "vec_data");

        opts.backup("Just a &str".with_name("str_data"));
        assert_eq!(opts.backup_data.len(), 4);
        assert_eq!(opts.backup_data[3].name, "str_data");

        opts.backup(1.with_name("num_data"));
        assert_eq!(opts.backup_data.len(), 5);
        assert_eq!(opts.backup_data[4].name, "num_data");
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
