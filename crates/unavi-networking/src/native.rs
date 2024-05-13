use anyhow::Result;

use xwt_core::endpoint::connect::{Connect, Connecting};
use xwt_wtransport::Connection;

pub async fn connect(addr: &str) -> Result<Connection> {
    let config = wtransport::ClientConfig::builder().with_bind_default();

    #[cfg(not(feature = "wtransport/dangerous-configuration"))]
    let config = config.with_native_certs();

    #[cfg(feature = "wtransport/dangerous-configuration")]
    let config = {
        tracing::warn!("WebTransport cert validation disabled for {}.", addr);
        config.with_no_cert_validation()
    };

    let endpoint = wtransport::Endpoint::client(config.build())?;
    let endpoint = xwt_wtransport::Endpoint(endpoint);

    let connecting = endpoint.connect(addr).await?;
    let session = connecting.wait_connect().await?;

    Ok(session)
}
