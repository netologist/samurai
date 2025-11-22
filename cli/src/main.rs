//! Command-line interface for the AI Agent Framework.
//!
//! This binary provides a CLI for interacting with AI agents. It supports:
//! - Single-turn mode: Process one query and exit
//! - REPL mode: Interactive conversation with the agent
//!
//! # Usage
//!
//! Single-turn mode:
//! ```bash
//! ai-agent --config config.yaml --query "What is 2+2?"
//! ```
//!
//! REPL mode:
//! ```bash
//! ai-agent --config config.yaml
//! ```
//!
//! Verbose logging:
//! ```bash
//! ai-agent --config config.yaml --verbose
//! ```

mod agent;
mod args;
mod repl;
mod single;

use agent::Agent;
use args::CliArgs;
use clap::Parser;
use colored::Colorize;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse command-line arguments
    let args = CliArgs::parse();

    // Enable verbose logging if requested
    // Load configuration
    let config = match &args.config {
        Some(path) => {
            if args.verbose {
                println!(
                    "{} {}",
                    "Loading configuration from:".bright_blue(),
                    path.display()
                );
            }
            config::load_from_file(path).map_err(|e| {
                eprintln!("{} {}", "Error:".bright_red().bold(), e);
                anyhow::anyhow!("Failed to load configuration: {}", e)
            })?
        }
        None => {
            if args.verbose {
                println!(
                    "{}",
                    "Loading configuration from default locations...".bright_blue()
                );
            }
            config::load_defaults().map_err(|e| {
                eprintln!("{} {}", "Error:".bright_red().bold(), e);
                anyhow::anyhow!("Failed to load configuration from defaults: {}", e)
            })?
        }
    };

    if args.verbose {
        println!(
            "{} {} with model {}",
            "Configuration loaded:".bright_blue(),
            config.llm.provider,
            config.llm.model
        );
    }

    // Validate configuration
    config::validate(&config).map_err(|e| {
        eprintln!("{} {}", "Configuration Error:".bright_red().bold(), e);
        anyhow::anyhow!("Invalid configuration: {}", e)
    })?;

    if args.verbose {
        println!("{}", "Configuration validated successfully".bright_green());
    }

    // Initialize agent
    let mut agent = Agent::new(config).map_err(|e| {
        eprintln!("{} {}", "Initialization Error:".bright_red().bold(), e);
        anyhow::anyhow!("Failed to initialize agent: {}", e)
    })?;

    if args.verbose {
        println!("{}", "Agent initialized successfully".bright_green());
    }

    // Branch to single-turn or REPL mode based on query argument
    match args.query {
        Some(query) => {
            if args.verbose {
                println!("{}", "Running in single-turn mode".bright_blue());
            }
            // Single-turn mode
            single::run(&mut agent, &query).await.map_err(|e| {
                eprintln!("{} {}", "Error:".bright_red().bold(), e);
                anyhow::anyhow!("Error processing query: {}", e)
            })?;
        }
        None => {
            if args.verbose {
                println!("{}", "Starting REPL mode".bright_blue());
            }
            // REPL mode
            repl::run(agent).await.map_err(|e| {
                eprintln!("{} {}", "REPL Error:".bright_red().bold(), e);
                anyhow::anyhow!("REPL error: {}", e)
            })?;
        }
    }

    Ok(())
}
