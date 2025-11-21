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
        Commands::Create  => commands::create::create_space(&actor).await?,
        Commands::Remove { id } => commands::remove::remove_space(&actor, &id).await?,
    }

    Ok(())
}
