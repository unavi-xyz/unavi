use anyhow::{anyhow, Result};
use xwt_core::endpoint::connect::{Connect, Connecting};
use xwt_web_sys::Session;

pub async fn connect(addr: &str) -> Result<Session> {
    let endpoint = xwt_web_sys::Endpoint::default();

    let connecting = endpoint.connect(addr).await.map_err(|e| anyhow!("{}", e))?;

    let session = connecting
        .wait_connect()
        .await
        .map_err(|e| anyhow!("{}", e))?;

    Ok(session)
}
