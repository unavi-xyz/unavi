use clap::Parser;
use dwn_server::DwnServerOptions;
use tracing::{Level, error};
use unavi_server::ServerOptions;
use xdid::methods::web::reqwest::Url;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Run a DWN server alongside the UNAVI server.
    #[arg(long, default_value_t = true)]
    dwn: bool,
    #[arg(long, default_value_t = 8080)]
    dwn_port: u16,
    #[arg(short, long, default_value_t = 5000)]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    if args.dwn {
        tokio::spawn(async move {
            if let Err(e) = dwn_server::run_server(DwnServerOptions {
                in_memory: false,
                port: args.dwn_port,
            })
            .await
            {
                error!("{e:?}");
            }
        });
    }

    let remote = if args.dwn {
        format!("http://localhost:{}", args.dwn_port)
    } else {
        unavi_constants::REMOTE_DWN_URL.to_string()
    };

    if let Err(e) = unavi_server::run_server(ServerOptions {
        in_memory: false,
        port: args.port,
        remote_dwn: Url::parse(&remote).unwrap(),
    })
    .await
    {
        error!("{e:?}");
    }
}
