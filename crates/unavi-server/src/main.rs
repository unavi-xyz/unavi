use std::{
    path::PathBuf,
    process::ExitCode,
};

use anyhow::Context;
use clap::Parser;
use directories::ProjectDirs;
use tracing::{
    Level,
    error,
};
use tracing_subscriber::{
    EnvFilter,
    fmt::writer::MakeWriterExt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use unavi_server::{
    ServerOptions,
    config::{
        Config,
        registry_config,
    },
};

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Enable debug logging.
    #[arg(long, default_value_t = false)]
    debug:       bool,
    /// Keeps the identity key and document store in memory, and hosts no
    /// files. Useful for running several servers on one machine.
    #[arg(long, default_value_t = false)]
    in_memory:   bool,
    /// Where keys, documents, hosted files and `registry.ron` live. Defaults
    /// to the platform's data directory.
    #[arg(long)]
    data_dir:    Option<PathBuf>,
    #[arg(short, long, default_value_t = 5000)]
    port:        u16,
    /// Do not serve discovery: catalog, curated views, and live presence.
    #[arg(long, default_value_t = false)]
    no_registry: bool,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();
    init_logging(args.debug);

    match run(args).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            error!(?err, "server failed");
            ExitCode::FAILURE
        }
    }
}

async fn run(args: Args) -> anyhow::Result<()> {
    let config = Config::load()?;

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

    unavi_server::run_server(
        ServerOptions {
            domain: config.unavi_domain,
            data_dir,
            port: args.port,
            registry,
        },
        shutdown_signal(),
    )
    .await
}

fn default_data_dir() -> anyhow::Result<PathBuf> {
    ProjectDirs::from("", "UNAVI", "unavi-server")
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

fn init_logging(debug: bool) {
    let level = if debug { Level::DEBUG } else { Level::INFO };

    let registry = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().map_writer(|w| w.with_max_level(level)));

    #[cfg(feature = "devtools-console")]
    let registry = registry.with(console_subscriber::spawn());

    registry
        .with(EnvFilter::from_default_env().add_directive(level.into()))
        .init();
}
