//! Command-line argument parsing for the AI agent CLI.

use clap::Parser;
use std::path::PathBuf;

/// Command-line arguments for the AI agent
#[derive(Parser, Debug)]
#[command(name = "ai-agent")]
#[command(about = "Educational AI Agent Framework", long_about = None)]
pub struct CliArgs {
    /// Path to configuration file (YAML format)
    /// If not provided, will search in standard locations
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Run in single-turn mode with this query (omit for REPL mode)
    #[arg(short, long)]
    pub query: Option<String>,

    /// Enable verbose logging for debugging
    #[arg(short, long)]
    pub verbose: bool,
}
