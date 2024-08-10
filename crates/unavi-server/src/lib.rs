use std::{
    path::PathBuf,
    sync::{Arc, LazyLock},
};

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use directories::ProjectDirs;
use dwn::{
    store::{DataStore, MessageStore},
    DWN,
};
use tracing::{debug, info_span, Instrument};

pub static STORAGE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let dirs =
        ProjectDirs::from("xyz", "unavi", "unavi-server").expect("Failed to get project dirs.");
    let path = dirs.data_dir().to_owned();
    std::fs::create_dir_all(&path).expect("Failed to create STORAGE_PATH");
    path
});

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Enables debug logging.
    #[arg(long)]
    pub debug: bool,

    #[arg(long, default_value = "filesystem")]
    pub storage: Storage,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Starts all servers with default settings.
    /// Useful for development.
    /// Will use a separate database for the combined server.
    All,
    /// Social server.
    /// Hosts a DWN, login APIs, and more.
    Social {
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
    /// World server.
    /// Hosts multiplayer instances of worlds.
    World {
        #[arg(short, long, default_value = "localhost:<port>")]
        domain: String,

        /// Remote DWN to connect to.
        #[arg(long, default_value = "http://localhost:3000")]
        remote_dwn: String,

        #[arg(short, long, default_value = "3001")]
        port: u16,

        /// Maximum number of threads to use for connection handling.
        /// Defaults to available parallelism.
        #[arg(short, long)]
        threads: Option<usize>,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Storage {
    Filesystem,
    Memory,
}

#[derive(Clone)]
pub struct StartOptions {
    pub enable_remote_sync: bool,
}

impl Default for StartOptions {
    fn default() -> Self {
        Self {
            enable_remote_sync: true,
        }
    }
}

#[async_recursion::async_recursion]
pub async fn start(
    args: Args,
    opts: StartOptions,
    dwn: Arc<DWN<impl DataStore + 'static, impl MessageStore + 'static>>,
) -> Result<()> {
    debug!("Args: {:?}", args);

    match args.command {
        Command::All => {
            let mut opts = opts.clone();
            opts.enable_remote_sync = false;

            tokio::select! {
                res = start(Args::parse_from(["unavi-server", "social"]), opts.clone(), dwn.clone()) => {
                    res?;
                }
                res = start(Args::parse_from(["unavi-server", "world"]), opts, dwn) => {
                    res?;
                }
            }
        }
        Command::Social { port } => {
            unavi_social_server::start(unavi_social_server::ServerOptions { dwn, port })
                .instrument(info_span!("Social"))
                .await?;
        }
        Command::World {
            domain,
            port,
            remote_dwn,
            threads,
        } => {
            let domain = if domain == "localhost:<port>" {
                format!("localhost:{}", port)
            } else {
                domain
            };

            let storage = match args.storage {
                Storage::Filesystem => unavi_world_host::Storage::Path(STORAGE_PATH.clone()),
                Storage::Memory => unavi_world_host::Storage::Memory,
            };

            let server_options = unavi_world_server::ServerOptions {
                domain: domain.clone(),
                dwn: dwn.clone(),
                port,
                threads,
            };

            let host_options = unavi_world_host::ServerOptions {
                domain,
                dwn,
                port,
                remote_dwn,
                remote_sync: opts.enable_remote_sync,
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
