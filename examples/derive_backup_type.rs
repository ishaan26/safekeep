use std::fs;

use safekeep::{BackupError, BackupType};
use serde::Serialize;

#[derive(BackupType, Serialize)]
#[safekeep(format = "yaml")]
struct MyData<'a> {
    name: String,
    age: u8,
    attrs: &'a [&'a str],
}

fn main() -> Result<(), BackupError> {
    let data = MyData {
        name: "Ishaan".to_string(),
        age: 26,
        attrs: &["h", "b", "c"],
    };

    let data = data.backup_bytes()?;
    fs::write("test.yaml", data)?;

    Ok(())
}
