use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DdSnapshotMeta {
    pub device: String,
    pub host: String,
    pub local_path: String,
    pub bytes_total: u64,
    pub bytes_written: u64,
    pub sha256: String,
    pub compression: String,
    pub finished_at: DateTime<Utc>,
}
