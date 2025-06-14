use crate::error::ConfigError;
use core::fmt;
use serde::{Deserialize, Deserializer};
use std::{fs, net::IpAddr, path::Path};
#[derive(Debug, Clone)]
pub enum Filesystem {
    Root,     // "/"
    RootHome, // "/root"
    Home,     // "/home"
    Etc,      // "/etc"
    Opt,      // "/opt"
    Srv,      // "/srv"
    Boot,     // "/boot"
    Mnt,      // "/mnt"
    Media,    // "/media"
    Usr,      // "/usr"
    Lib,      // "/lib"
    Lib64,    // "/lib64"
    Bin,      // "/bin"
    Sbin,     // "/sbin"
    Tmp,      // "/tmp"
    Var,      // "/var"
    VarWww,   // "/var/www"
    Dev,      // "/dev"
    Proc,     // "/proc"
    Sys,      // "/sys"
    Custom(String),
}

impl<'de> Deserialize<'de> for Filesystem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        let fs = match s.as_str() {
            "/" => Filesystem::Root,
            "/root" => Filesystem::RootHome,
            "/home" => Filesystem::Home,
            "/etc" => Filesystem::Etc,
            "/opt" => Filesystem::Opt,
            "/srv" => Filesystem::Srv,
            "/boot" => Filesystem::Boot,
            "/mnt" => Filesystem::Mnt,
            "/media" => Filesystem::Media,
            "/usr" => Filesystem::Usr,
            "/lib" => Filesystem::Lib,
            "/lib64" => Filesystem::Lib64,
            "/bin" => Filesystem::Bin,
            "/sbin" => Filesystem::Sbin,
            "/tmp" => Filesystem::Tmp,
            "/var" => Filesystem::Var,
            "/var/www" => Filesystem::VarWww,
            "/dev" => Filesystem::Dev,
            "/proc" => Filesystem::Proc,
            "/sys" => Filesystem::Sys,
            other => Filesystem::Custom(other.to_string()),
        };
        Ok(fs)
    }
}

impl fmt::Display for Filesystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Filesystem::Root => "/",
            Filesystem::RootHome => "/root",
            Filesystem::Home => "/home",
            Filesystem::Etc => "/etc",
            Filesystem::Opt => "/opt",
            Filesystem::Srv => "/srv",
            Filesystem::Boot => "/boot",
            Filesystem::Mnt => "/mnt",
            Filesystem::Media => "/media",
            Filesystem::Usr => "/usr",
            Filesystem::Lib => "/lib",
            Filesystem::Lib64 => "/lib64",
            Filesystem::Bin => "/bin",
            Filesystem::Sbin => "/sbin",
            Filesystem::Tmp => "/tmp",
            Filesystem::Var => "/var",
            Filesystem::VarWww => "/var/www",
            Filesystem::Dev => "/dev",
            Filesystem::Proc => "/proc",
            Filesystem::Sys => "/sys",
            Filesystem::Custom(s) => s,
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub mode: String,
    /// List of filesystem paths to include in the snapshot (e.g. `/root`, `/var/www`).
    pub filesystems: Vec<Filesystem>,
    /// Remote SSH connection parameters.
    pub remote: Remote,
    /// Backup-file naming & target directory.
    pub backup: Backup,
    /// Optional feature toggles.
    pub options: Options,

    #[serde(default)]
    pub dd: Option<DdConfig>,
}

#[derive(Debug, Deserialize)]
pub struct Remote {
    pub host: IpAddr,

    #[serde(default = "Remote::default_port")]
    pub port: u16,

    #[serde(default = "Remote::default_user")]
    pub user: String,

    pub password: String,

    #[serde(default)]
    pub private_key: Option<String>,
}
impl Remote {
    fn default_port() -> u16 {
        22
    }
    fn default_user() -> String {
        "root".into()
    }
}

#[derive(Debug, Deserialize)]
pub struct Backup {
    /// Remote directory where the archive will be created.
    pub dir: String,
    /// Archive filename
    pub filename: String,
}

#[derive(Debug, Deserialize)]
pub struct Options {
    /// Default is false. We only construct the snapshot without local download.
    #[serde(default)]
    pub download_to_local: bool,
    #[serde(default = "Options::default_download_dir")]
    pub local_download_dir: String,
}
impl Options {
    fn default_download_dir() -> String {
        ".".to_string()
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            download_to_local: false,
            local_download_dir: Self::default_download_dir(),
        }
    }
}

pub fn load<P: AsRef<Path>>(path: P) -> Result<Config, ConfigError> {
    let raw = fs::read_to_string(&path).map_err(|e| ConfigError::Validation(e.to_string()))?;
    let cfg: Config = toml::from_str(&raw)?;

    if cfg.filesystems.is_empty() {
        panic!("`filesystems` list must not be empty");
    }
    if cfg.backup.filename.trim().is_empty() {
        panic!("`backup.filename` must not be empty"); //   panic
    }
    if cfg.remote.password.trim().is_empty() {
        panic!("`remote.password` must not be empty"); //  panic
    }

    Ok(cfg)
}
#[derive(Debug, Deserialize)]
pub struct DdConfig {
    /// Can be: `/dev/vda`, `UUID=...`, or `SERIAL=...`
    pub device: String,

    /// Block size in bytes. Default: 65536
    #[serde(default = "DdConfig::default_block_size")]
    pub block_size: u64,

    /// "none" | "gzip" | "zstd" | "xz"
    #[serde(default = "DdConfig::default_compression")]
    pub compression: String,

    /// "fresh" | "continue"
    #[serde(default = "DdConfig::default_resume")]
    pub resume: String,

    /// If true, runs `sudo dd` remotely
    #[serde(default = "DdConfig::default_sudo")]
    pub sudo: bool,
}

impl DdConfig {
    fn default_block_size() -> u64 {
        65536
    }
    fn default_compression() -> String {
        "none".into()
    }
    fn default_resume() -> String {
        "fresh".into()
    }
    fn default_sudo() -> bool {
        true
    }
}
