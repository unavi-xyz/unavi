use std::{
    path::PathBuf,
    process::ExitCode,
};

use anyhow::Context;
use clap::Parser;
use directories::ProjectDirs;
use tracing::error;
use unavi_node::{
    NodeOptions,
    config::registry_config,
};

mod logging;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Enable debug logging.
    #[arg(long, default_value_t = false)]
    debug:       bool,
    /// Keeps the identity key and document store in memory, and hosts no
    /// files. Useful for running several nodes on one machine.
    #[arg(long, default_value_t = false)]
    in_memory:   bool,
    /// Where keys, documents, hosted files and `registry.ron` live. Defaults
    /// to the platform's data directory.
    #[arg(long)]
    data_dir:    Option<PathBuf>,
    /// Domain the node's `did:web` identity resolves from. Defaults to
    /// `localhost:<port>`.
    #[arg(long)]
    domain:      Option<String>,
    #[arg(short, long, default_value_t = 5000)]
    port:        u16,
    /// Do not serve discovery: catalog, curated views, and live presence.
    #[arg(long, default_value_t = false)]
    no_registry: bool,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();
    logging::init(args.debug);

    match run(args).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            error!(?err, "node failed");
            ExitCode::FAILURE
        }
    }
}

async fn run(args: Args) -> anyhow::Result<()> {
    let domain = args
        .domain
        .unwrap_or_else(|| format!("localhost:{}", args.port));

    let data_dir = if args.in_memory {
        None
    } else {
        Some(match args.data_dir {
            Some(dir) => dir,
            None => default_data_dir()?,
        })
    };
    if let Some(dir) = &data_dir {
        std::fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
    }

    let registry = if args.no_registry {
        None
    } else {
        Some(registry_config(data_dir.as_deref())?)
    };

    unavi_node::run(
        NodeOptions {
            domain,
            data_dir,
            port: args.port,
            registry,
        },
        shutdown_signal(),
    )
    .await
}

fn default_data_dir() -> anyhow::Result<PathBuf> {
    ProjectDirs::from("", "UNAVI", "unavi-node")
        .map(|dirs| dirs.data_local_dir().to_path_buf())
        .context("no home directory to keep data in; pass --data-dir")
}

/// Resolves on Ctrl-C, or on SIGTERM where there is one, which is how systemd
/// stops the service.
async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(err) = tokio::signal::ctrl_c().await {
            error!(?err, "cannot listen for Ctrl-C");
            std::future::pending::<()>().await;
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(err) => {
                error!(?err, "cannot listen for SIGTERM");
                std::future::pending::<()>().await;
            }
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }
}
