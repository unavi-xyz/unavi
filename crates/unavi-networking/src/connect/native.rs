use anyhow::Result;

use wtransport::ClientConfig;
use xwt_core::endpoint::connect::{Connect, Connecting};
use xwt_wtransport::Connection;

pub async fn connect(addr: &str) -> Result<Connection> {
    let config = ClientConfig::builder().with_bind_default();

    let config = if addr.starts_with("https://127.0.0.1:") {
        config.with_no_cert_validation()
    } else {
        config.with_native_certs()
    };

    let endpoint = wtransport::Endpoint::client(config.build())?;
    let endpoint = xwt_wtransport::Endpoint(endpoint);

    let connecting = endpoint.connect(addr).await?;
    let session = connecting.wait_connect().await?;

    Ok(session)
}
