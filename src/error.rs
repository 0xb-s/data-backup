use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("configuration error: {0}")]
    Config(#[from] ConfigError),
    #[error("SSH error: {0}")]
    Ssh(#[from] ssh2::Error),
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("remote command exit status {0}")]
    RemoteExit(i32),
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("cannot read config file: {0}")]
    Io(#[from] io::Error),
    #[error("TOML syntax error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("{0}")]
    Validation(String),
}
