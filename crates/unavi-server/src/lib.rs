use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use tracing::{debug, info_span, Instrument};

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Enables debug logging.
    #[arg(long)]
    pub debug: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Starts all servers with default settings.
    /// Useful for development.
    All,
    /// Social server.
    /// Hosts a DWN, login APIs, and more.
    Social {
        #[arg(short, long, default_value = "localhost:<port>")]
        domain: String,

        /// Path to store data in.
        #[arg(long, default_value = ".unavi/server/social")]
        path: String,

        #[arg(short, long, default_value = "3000")]
        port: u16,

        #[arg(long, default_value = "filesystem")]
        storage: Storage,
    },
    /// World server.
    /// Hosts multiplayer instances of worlds.
    World {
        #[arg(short, long, default_value = "localhost:<port>")]
        domain: String,

        /// The DWN to use for the world host.
        #[arg(long, default_value = "http://localhost:3000")]
        dwn_url: String,

        /// Path to store data in.
        #[arg(long, default_value = ".unavi/server/world")]
        path: String,

        #[arg(short, long, default_value = "3001")]
        port: u16,

        #[arg(long, default_value = "filesystem")]
        storage: Storage,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Storage {
    Filesystem,
    Memory,
}

#[async_recursion::async_recursion]
pub async fn start(args: Args) -> Result<()> {
    debug!("Processing args: {:?}", args);

    match args.command {
        Command::All => {
            tokio::select! {
                res = start(Args::parse_from(["unavi-server", "social"])) => {
                    res?;
                }
                res = start(Args::parse_from(["unavi-server", "world"])) => {
                    res?;
                }
            }
        }
        Command::Social {
            domain,
            path,
            port,
            storage,
        } => {
            let domain = if domain == "localhost:<port>" {
                format!("localhost:{}", port)
            } else {
                domain
            };

            let storage = match storage {
                Storage::Filesystem => unavi_social_server::Storage::Path(path),
                Storage::Memory => unavi_social_server::Storage::Memory,
            };

            unavi_social_server::start(unavi_social_server::ServerOptions {
                domain,
                port,
                storage,
            })
            .instrument(info_span!("Social"))
            .await?;
        }
        Command::World {
            domain,
            dwn_url,
            path,
            port,
            storage,
        } => {
            let domain = if domain == "localhost:<port>" {
                format!("localhost:{}", port)
            } else {
                domain
            };

            let server_options = unavi_world_server::ServerOptions {
                port,
                domain: domain.clone(),
            };

            let storage = match storage {
                Storage::Filesystem => unavi_world_host::Storage::Path(path),
                Storage::Memory => unavi_world_host::Storage::Memory,
            };

            let host_options = unavi_world_host::ServerOptions {
                domain,
                dwn_url,
                port,
                storage,
            };

            let span = info_span!("World");

            tokio::select! {
                res = unavi_world_server::start(server_options).instrument(info_span!(parent: &span, "Server")) => {
                    res?;
                }
                res = unavi_world_host::start(host_options).instrument(info_span!(parent: &span, "Host")) => {
                    res?;
                }
            };
        }
    };

    Ok(())
}
