use safekeep::{BackupError, BackupOptions, BackupType};
use serde::Serialize;

#[derive(BackupType, Serialize)]
#[safekeep(format = "toml")]
struct MyData<'a> {
    name: String,
    age: u8,
    attrs: &'a [&'a str],
}

fn main() -> Result<(), BackupError> {
    let a = vec![1, 2, 3, 4, 5, 6, 7];
    let b = MyData {
        name: "Ishaan".to_string(),
        age: 26,
        attrs: &["h", "b", "c"],
    };
    let c = "This is a test str";

    BackupOptions::new()
        .path("./test_files")
        .backup_data(a)
        .backup_data(b)
        .backup_data(c)
        .run()?;

    Ok(())
}
