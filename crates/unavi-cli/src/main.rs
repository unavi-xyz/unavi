use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod identity;

#[derive(Parser, Debug)]
#[command(version)]
#[command(about = "UNAVI CLI - Manage spaces in your DWN", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List all spaces from your DWN
    List,
    /// Create a new space
    Create,
    /// Edit an existing space with JSON data from a file
    Edit {
        /// ID of the space to edit
        id: String,
        /// Path to JSON file containing space data
        data_path: PathBuf,
    },
    /// Remove a space by ID
    Remove {
        /// ID of the space to remove
        id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    let actor = identity::init_actor().await?;

    match args.command {
        Commands::List => commands::list::list_spaces(&actor).await?,
        Commands::Create => commands::create::create_space(&actor).await?,
        Commands::Edit { id, data_path } => {
            commands::edit::edit_space(&actor, id, data_path).await?
        }
        Commands::Remove { id } => commands::remove::remove_space(&actor, id).await?,
    }

    Ok(())
}
