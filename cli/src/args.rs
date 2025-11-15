use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "ai-agent")]
#[command(about = "Educational AI Agent Framework", long_about = None)]
pub struct CliArgs {
    /// Path to configuration file
    #[arg(short, long)]
    pub config: PathBuf,

    /// Run in single-turn mode with this query
    #[arg(short, long)]
    pub query: Option<String>,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,
}
