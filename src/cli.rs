use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Args {
    /// Path to `config.toml`.
    #[arg(short = 'c', long, default_value = "config.toml")]
    pub config: String,

    #[arg(short, long)]
    pub verbose: bool,
}
