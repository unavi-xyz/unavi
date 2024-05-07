use anyhow::Result;
use xwt_core::{
    base::Session,
    endpoint::connect::{Connect, Connecting},
};

pub async fn connect(addr: &str) -> Result<impl Session> {
    let endpoint = xwt_web_sys::Endpoint::default();

    let connecting = endpoint.connect(addr).await?;

    let session = connecting.wait_connect().await?;

    Ok(session)
}
