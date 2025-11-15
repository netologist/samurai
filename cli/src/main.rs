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
    if args.verbose {
        println!("{}", "Verbose logging enabled".bright_blue());
        println!("{} {}", "Loading configuration from:".bright_blue(), args.config.display());
    }

    // Load configuration from file
    let config = config::load_from_file(&args.config).map_err(|e| {
        eprintln!("{} {}", "Error:".bright_red().bold(), e);
        anyhow::anyhow!("Failed to load configuration: {}", e)
    })?;

    if args.verbose {
        println!("{} {} with model {}", 
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
