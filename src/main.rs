#![allow(dead_code)]

mod backup;
mod cli;
mod config;
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
    backup::run(&cfg)?;
    Ok(())
}
