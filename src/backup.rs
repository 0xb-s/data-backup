use crate::{config::Config, error::AppError, ssh::Ssh};
use chrono::{SecondsFormat, Utc};
use std::path::PathBuf;

pub fn run(cfg: &Config) -> Result<(), AppError> {
    log::info!("Connecting to {}", cfg.remote.host);
    let ssh =
        Ssh::connect(cfg.remote.host, cfg.remote.port, &cfg.remote.user, &cfg.remote.password)?;

    let filename = resolve_filename(&cfg.backup.filename);
    let remote_path = format!("{}/{}", cfg.backup.dir.trim_end_matches('/'), filename);

    let paths = cfg.filesystems.iter().map(|fs| fs.to_string()).collect::<Vec<_>>().join(" ");

    let cmd = format!(
        "sudo tar \
        --exclude=/proc \
        --exclude=/sys \
        --exclude=/dev \
        --exclude=/run \
        --exclude=/mnt \
        --exclude=/media \
        --exclude=/tmp \
        --exclude=/var/run \
        --exclude=/var/lock \
        --exclude=/lost+found \
        --ignore-failed-read \
        -czpvf '{}' {}",
        remote_path, paths
    );

    log::info!("Remote tar command: {}", cmd);

    log::info!("Creating snapshot on remote: {}", cmd);
    ssh.exec_verbose(&cmd)?;

    log::info!("Snapshot created at {}", remote_path);

    if cfg.options.download_to_local {
        let mut local_path = PathBuf::from(&cfg.options.local_download_dir);
        std::fs::create_dir_all(&local_path)?;
        local_path.push(&filename);

        ssh.download(&remote_path, &local_path)?;
        log::info!("Snapshot saved to {:?}", local_path);
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
