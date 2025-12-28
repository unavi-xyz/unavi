use clap::Parser;
use tracing::{Level, error};
use tracing_subscriber::{
    fmt::writer::MakeWriterExt, layer::SubscriberExt, util::SubscriberInitExt,
};
use unavi_server::ServerOptions;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Enable debug logging.
    #[arg(long, default_value_t = false)]
    debug: bool,
    #[arg(short, long, default_value_t = 5000)]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let registry = tracing_subscriber::registry();

    let level = if args.debug {
        Level::DEBUG
    } else {
        Level::INFO
    };
    let registry =
        registry.with(tracing_subscriber::fmt::layer().map_writer(|w| w.with_max_level(level)));

    #[cfg(feature = "devtools-console")]
    let registry = registry.with(console_subscriber::spawn());

    registry.init();

    if let Err(e) = unavi_server::run_server(ServerOptions {
        in_memory: false,
        port: args.port,
    })
    .await
    {
        error!("{e:?}");
    }
}
