use anyhow::Result;
use xwt_core::{
    base::Session,
    endpoint::connect::{Connect, Connecting},
};

pub async fn connect(addr: &str) -> Result<impl Session> {
    let config = wtransport::ClientConfig::builder()
        .with_bind_default()
        .with_native_certs()
        .build();

    let endpoint = wtransport::Endpoint::client(config)?;
    let endpoint = xwt_wtransport::Endpoint(endpoint);

    let connecting = endpoint.connect(addr).await?;
    let session = connecting.wait_connect().await?;

    Ok(session)
}
