//! Runs the remote dd copy, verifies hash, writes metadata JSON.

use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

use chrono::Utc;
use indicatif::{ProgressBar, ProgressStyle};
use sha2::{Digest, Sha256};

use super::{ResumeMode, meta::DdSnapshotMeta};
use crate::{
    dd::builder::DdSnapshotConfig,
    error::AppError,
    metadata::generator::{BackupMeta, JsonWriter},
};

/// Convenience wrapper if you just want to run immediately.
pub fn run_once(cfg: DdSnapshotConfig) -> Result<DdSnapshotMeta, AppError> {
    DdPipeline::new(cfg).run()
}

pub struct DdPipeline {
    cfg: DdSnapshotConfig,
}

impl DdPipeline {
    pub fn new(cfg: DdSnapshotConfig) -> Self {
        Self { cfg }
    }

    pub fn run(self) -> Result<DdSnapshotMeta, AppError> {
        let cfg = self.cfg;

        // -- 1. fetch device size ---------------------------------------------
        let size_cmd = format!(
            "{sudo}blockdev --getsize64 {}",
            cfg.device.dev_path(),
            sudo = if cfg.sudo { "sudo " } else { "" }
        );
        let mut buf = Vec::<u8>::new();
        cfg.ssh.exec_capture(&size_cmd, &mut buf)?;
        let dev_size: u64 = String::from_utf8_lossy(&buf).trim().parse().unwrap_or(0);

        // -- 2. prepare remote dd command -------------------------------------
        let dd_cmd = format!(
            r#"{sudo}dd if={} bs={} iflag=fullblock,noatime status=progress | {}"#,
            cfg.device.dev_path(),
            cfg.block_size,
            cfg.compression.pipe(),
            sudo = if cfg.sudo { "sudo " } else { "" },
        );

        // -- 3. local file (resume / fresh) -----------------------------------
        let (mut file, offset) = match cfg.resume_mode {
            ResumeMode::Fresh => (File::create(&cfg.local_path)?, 0),
            ResumeMode::Continue => {
                let f = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&cfg.local_path)?;
                (f.try_clone()?, f.metadata()?.len())
            }
        };

        // -- 4. open remote channel -------------------------------------------
        let mut ch = cfg.ssh.open_stream(&dd_cmd)?;

        // -- 5. copy + hash ----------------------------------------------------
        let mut hasher = Sha256::new();
        let pb = ProgressBar::new(dev_size.saturating_sub(offset));
        pb.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] {wide_bar} {bytes}/{total_bytes} ({bytes_per_sec})",
            )
            .unwrap(),
        );

        let mut buf64 = [0u8; 1 << 20];
        let mut written = offset;
        loop {
            let n = ch.read(&mut buf64)?;
            if n == 0 {
                break;
            }
            hasher.update(&buf64[..n]);
            file.write_all(&buf64[..n])?;
            written += n as u64;
            pb.inc(n as u64);
        }
        pb.finish();
        ch.wait_close()?;
        if ch.exit_status()? != 0 {
            return Err(AppError::RemoteExit(ch.exit_status()?));
        }

        let sha_hex = hex::encode(hasher.finalize());

        // -- 6. metadata -------------------------------------------------------
        let meta = DdSnapshotMeta {
            device: cfg.device.dev_path(),
            host: cfg.ssh.remote_addr_string(),
            local_path: cfg.local_path.to_string_lossy().into_owned(),
            bytes_total: dev_size,
            bytes_written: written,
            sha256: sha_hex.clone(),
            compression: format!("{:?}", cfg.compression),
            finished_at: Utc::now(),
        };

        JsonWriter::write(
            &BackupMeta {
                snapshot_name: Path::new(&meta.local_path)
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                remote_path: meta.device.clone(),
                local_path: meta.local_path.clone(),
                size_bytes: meta.bytes_total,
                sha256: sha_hex,
                timestamp: meta.finished_at,
                filesystems: vec![],
            },
            Path::new("metadata"),
        )?;

        Ok(meta)
    }
}
