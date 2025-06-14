use serde::Deserialize;

use crate::{error::AppError, ssh::Ssh};

#[derive(Debug, Deserialize, Clone)]
pub struct BlockDevice {
    pub name: String,
    pub serial: Option<String>,
    pub uuid: Option<String>,
    pub size: String,
    pub mountpoint: Option<String>,
}
#[derive(Debug, Deserialize)]
struct LsblkJson {
    blockdevices: Vec<BlockDevice>,
}

impl BlockDevice {
    pub fn dev_path(&self) -> String {
        format!("/dev/{}", self.name)
    }
}

/// Runs `lsblk` on the remote host and returns the parsed list.
pub fn remote_lsblk(ssh: &Ssh, sudo: bool) -> Result<Vec<BlockDevice>, AppError> {
    let cmd = if sudo {
        "sudo lsblk -lJ -o NAME,SERIAL,UUID,SIZE,MOUNTPOINT"
    } else {
        "lsblk -lJ -o NAME,SERIAL,UUID,SIZE,MOUNTPOINT"
    };
    let mut ch = ssh.open_stream(cmd)?;
    let mut json = String::new();
    use std::io::Read;
    ch.read_to_string(&mut json)?;
    ch.wait_close()?;
    if ch.exit_status()? != 0 {
        return Err(AppError::RemoteExit(ch.exit_status()?));
    }
    let parsed: LsblkJson =
        serde_json::from_str(&json).map_err(|e| AppError::Remote(format!("lsblk json: {e}")))?;
    Ok(parsed.blockdevices)
}
