mod backup;
mod cli;
mod config;
mod error;
mod ssh;

use clap::Parser;
use cli::Args;
use log::LevelFilter;

use crate::error::AppError;

fn main() -> Result<(), AppError> {

    env_logger::Builder::new().filter_level(log::LevelFilter::Info).init();

    let args = Args::parse();
    let cfg = config::load(&args.config)?;
    backup::run(&cfg)?;
    Ok(())
}
