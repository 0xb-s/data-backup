//! Turn `Config` + `[dd]` toml table into `DdSnapshotConfig`.

use std::{path::PathBuf, time::Duration};

use crate::{config::Config, error::AppError, ssh::Ssh};

use chrono::Utc;

use super::probe::{BlockDevice, remote_lsblk};

/// When an existing local file is present.
#[derive(Debug, Clone, Copy)]
pub enum ResumeMode {
    Fresh,
    Continue,
}

/// Pipe compression.
#[derive(Debug, Clone, Copy)]
pub enum Compression {
    None,
    Gzip,
    Zstd,
    Xz,
}
impl Compression {
    pub fn parse(txt: &str) -> Self {
        match txt.to_ascii_lowercase().as_str() {
            "gzip" => Self::Gzip,
            "zstd" => Self::Zstd,
            "xz" => Self::Xz,
            _ => Self::None,
        }
    }
    pub fn ext(self) -> &'static str {
        match self {
            Self::None => ".img",
            Self::Gzip => ".img.gz",
            Self::Zstd => ".img.zst",
            Self::Xz => ".img.xz",
        }
    }
    pub fn pipe(self) -> &'static str {
        match self {
            Self::None => "cat",
            Self::Gzip => "gzip -c",
            Self::Zstd => "zstd -q -c",
            Self::Xz => "xz -c",
        }
    }
}

/// Immutable job configuration for the pipeline.
#[derive(Debug)]
pub struct DdSnapshotConfig {
    pub ssh: Ssh,
    pub device: BlockDevice,
    pub compression: Compression,
    pub block_size: u64,
    pub resume_mode: ResumeMode,
    pub sudo: bool,
    pub local_path: PathBuf,
    pub read_to: Duration,
    pub write_to: Duration,
}

/// Builds from `config.toml`.
pub struct DdBuilder<'a> {
    cfg: &'a Config,
}

impl<'a> DdBuilder<'a> {
    pub fn new(cfg: &'a Config) -> Self {
        Self { cfg }
    }

    pub fn build(self) -> Result<DdSnapshotConfig, AppError> {
        // 1. SSH connect -------------------------------------------------------
        let r = &self.cfg.remote;
        let ssh = Ssh::connect(
            r.host,
            r.port,
            &r.user,
            &r.password,
            r.private_key.as_deref(),
        )?;

        // 2. Read [dd] config safely ------------------------------------------
        let dd_cfg = self.cfg.dd.as_ref();

        let dev_query = dd_cfg.map(|c| c.device.as_str()).unwrap_or("/dev/vda");

        let block_size = dd_cfg.map(|c| c.block_size).unwrap_or(64 * 1024);

        let compression =
            Compression::parse(dd_cfg.map(|c| c.compression.as_str()).unwrap_or("none"));

        let resume_mode = match dd_cfg
            .map(|c| c.resume.as_str())
            .unwrap_or("fresh")
            .to_ascii_lowercase()
            .as_str()
        {
            "continue" => ResumeMode::Continue,
            _ => ResumeMode::Fresh,
        };

        let sudo = dd_cfg.map(|c| c.sudo).unwrap_or(true);

        // 3. Remote lsblk -----------------------------------------------------
        let devices = remote_lsblk(&ssh, sudo)?;
        let dev = select_device(dev_query, &devices)
            .ok_or_else(|| AppError::Validation(format!("Device `{dev_query}` not found")))?;

        // 4. Local filename ---------------------------------------------------
        let mut path = std::path::PathBuf::from(&self.cfg.options.local_download_dir);
        std::fs::create_dir_all(&path)?;
        path.push(Utc::now().format("%Y%m%dT%H%M%SZ").to_string());
        path.set_extension(compression.ext().trim_start_matches('.'));

        Ok(DdSnapshotConfig {
            ssh,
            device: dev.clone(),
            compression,
            block_size,
            resume_mode,
            sudo,
            local_path: path,
            read_to: std::time::Duration::from_secs(120),
            write_to: std::time::Duration::from_secs(120),
        })
    }
}

fn select_device<'d>(q: &str, list: &'d [BlockDevice]) -> Option<&'d BlockDevice> {
    if q.starts_with("/dev/") {
        return list.iter().find(|b| b.dev_path() == q);
    }
    if let Some(x) = q.strip_prefix("UUID=") {
        return list.iter().find(|b| b.uuid.as_deref() == Some(x));
    }
    if let Some(x) = q.strip_prefix("SERIAL=") {
        return list.iter().find(|b| b.serial.as_deref() == Some(x));
    }
    None
}
