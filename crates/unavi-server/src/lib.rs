use std::net::SocketAddr;

#[cfg(feature = "web")]
mod axum_server;

#[cfg(feature = "world")]
mod world_server;

#[derive(Clone)]
pub struct ServerOptions {
    pub address: SocketAddr,
}

impl Default for ServerOptions {
    fn default() -> Self {
        Self {
            address: SocketAddr::from(([127, 0, 0, 1], 3000)),
        }
    }
}

pub async fn start(opts: ServerOptions) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "web")]
    {
        let opts = opts.clone();
        tokio::spawn(async move {
            if let Err(e) = axum_server::start(&opts).await {
                tracing::error!(e);
                panic!("axum_server failed");
            }
        });
    }

    #[cfg(feature = "world")]
    {
        let opts = opts.clone();
        tokio::spawn(async move {
            if let Err(e) = world_server::start(&opts).await {
                tracing::error!(e);
                panic!("world_server failed");
            }
        });
    }

    Ok(())
}
