use anyhow::Result;

use xwt_core::endpoint::connect::{Connect, Connecting};
use xwt_wtransport::Connection;

pub async fn connect(addr: &str) -> Result<Connection> {
    let config = wtransport::ClientConfig::builder()
        .with_bind_default()
        .with_native_certs();

    let endpoint = wtransport::Endpoint::client(config.build())?;
    let endpoint = xwt_wtransport::Endpoint(endpoint);

    let connecting = endpoint.connect(addr).await?;
    let session = connecting.wait_connect().await?;

    Ok(session)
}
