use clap::{Parser, Subcommand, ValueEnum};
use tracing::{error, info_span, Instrument};

#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    /// Enables debug logging.
    #[arg(long)]
    pub debug: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
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

#[derive(ValueEnum, Clone)]
pub enum Storage {
    Filesystem,
    Memory,
}

pub async fn start(args: Args) {
    match args.command {
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

            if let Err(e) = unavi_social_server::start(unavi_social_server::ServerOptions {
                domain,
                port,
                storage,
            })
            .await
            {
                error!("{}", e);
            };
        }
        Command::World {
            domain,
            dwn_url,
            path,
            port,
            storage,
        } => {
            let span_world_server = info_span!("world server");
            let span_world_host = info_span!("world host");

            let results = tokio::join!(
                // Run world server on UDP.
                {
                    let domain = domain.clone();
                    tokio::spawn(
                        async move {
                            unavi_world_server::start(unavi_world_server::ServerOptions {
                                port,
                                domain,
                            })
                            .await
                        }
                        .instrument(span_world_server),
                    )
                },
                // Run world host on TCP.
                tokio::spawn(
                    async move {
                        let domain = if domain == "localhost:<port>" {
                            format!("localhost:{}", port)
                        } else {
                            domain
                        };

                        let storage = match storage {
                            Storage::Filesystem => unavi_world_host::Storage::Path(path),
                            Storage::Memory => unavi_world_host::Storage::Memory,
                        };

                        unavi_world_host::start(unavi_world_host::ServerOptions {
                            domain,
                            dwn_url,
                            port,
                            storage,
                        })
                        .await
                    }
                    .instrument(span_world_host)
                )
            );

            let results = [results.0, results.1];

            for result in results {
                match result {
                    Ok(Ok(_)) => {}
                    Ok(Err(e)) => error!("{}", e),
                    Err(e) => error!("{}", e),
                }
            }
        }
    }
}
