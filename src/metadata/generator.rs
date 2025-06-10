use chrono::{DateTime, Utc};
use serde::Serialize;
use std::{fs, io, path::Path};

#[derive(Debug, Serialize)]
pub struct BackupMeta {
    pub snapshot_name: String,
    pub remote_path: String,
    pub local_path: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub timestamp: DateTime<Utc>,
    pub filesystems: Vec<String>,
}

pub struct JsonWriter;

impl JsonWriter {
    pub fn write<P: AsRef<Path>>(meta: &BackupMeta, dir: P) -> io::Result<()> {
        fs::create_dir_all(&dir)?;
        let mut json_path = dir.as_ref().to_path_buf();
        json_path.push(format!("{}.json", meta.snapshot_name));
        let json = serde_json::to_string_pretty(meta)?;
        fs::write(json_path, json)?;
        Ok(())
    }
}
