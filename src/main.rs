#![allow(dead_code)]
mod backup;
mod cli;
mod config;
mod dd;
mod error;
mod metadata;
mod ssh;
mod tar;
use clap::Parser;
use cli::Args;

use crate::error::AppError;
fn main() -> Result<(), AppError> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();

    let args = Args::parse();
    let cfg = config::load(&args.config)?;

    match cfg.mode.as_str() {
        "dd" => backup::run_dd(&cfg)?,
        "tar" => backup::run(&cfg)?,
        other => return Err(AppError::Validation(format!("Invalid mode: {other}"))),
    }

    Ok(())
}
