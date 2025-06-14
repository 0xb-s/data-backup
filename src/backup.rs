use crate::{
    config::Config,
    dd::{DdBuilder, run_once},
    error::AppError,
    metadata::{BackupMeta, JsonWriter},
    ssh::Ssh,
    tar::{Compression, TarBuilder, verify::sha256_file},
};

use chrono::{SecondsFormat, Utc};
use std::path::PathBuf;
pub fn run(cfg: &Config) -> Result<(), AppError> {
    log::info!("Connecting to {}", cfg.remote.host);
    let ssh = Ssh::connect(
        cfg.remote.host,
        cfg.remote.port,
        &cfg.remote.user,
        &cfg.remote.password,
        cfg.remote.private_key.as_deref(),
    )?;

    let filename = resolve_filename(&cfg.backup.filename);
    let remote_path = format!("{}/{}", cfg.backup.dir.trim_end_matches('/'), filename);

    let tar_cmd = TarBuilder::new(&remote_path)
        .paths(cfg.filesystems.iter().map(|fs| fs.to_string()))
        .exclude_default_runtime()
        .compression(Compression::Gzip)
        .build()
        .unwrap();

    log::info!("Creating snapshot on remote: {}", tar_cmd);
    ssh.exec_verbose(&tar_cmd)?;

    log::info!("Snapshot created at {}", remote_path);

    if cfg.options.download_to_local {
        let mut local_path = PathBuf::from(&cfg.options.local_download_dir);
        std::fs::create_dir_all(&local_path)?;
        local_path.push(&filename);

        ssh.download(&remote_path, &local_path)?;
        log::info!("Snapshot saved to {:?}", local_path);

        let metadata = BackupMeta {
            snapshot_name: filename.clone(),
            remote_path: remote_path.clone(),
            local_path: local_path.display().to_string(),
            size_bytes: std::fs::metadata(&local_path)?.len(),
            sha256: sha256_file(&local_path)?,
            timestamp: Utc::now(),
            filesystems: cfg.filesystems.iter().map(ToString::to_string).collect(),
        };

        JsonWriter::write(&metadata, &cfg.options.local_download_dir)?;

        log::info!("Metadata saved to {:?}", cfg.options.local_download_dir);
    }

    Ok(())
}

fn resolve_filename(template: &str) -> String {
    if template.contains("{{timestamp}}") {
        let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
        template.replace("{{timestamp}}", &ts)
    } else {
        template.into()
    }
}

pub fn run_dd(cfg: &Config) -> Result<(), AppError> {
    log::info!("Running dd snapshot from config");

    let dd_cfg = DdBuilder::new(cfg).build()?;
    let meta = run_once(dd_cfg)?;

    log::info!("Snapshot saved to {}", meta.local_path);
    Ok(())
}
