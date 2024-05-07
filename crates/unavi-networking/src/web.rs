use xwt_core::endpoint::connect::{Connect, Connecting};
use xwt_web_sys::{Error, Session};

pub async fn connect(addr: &str) -> Result<Session, Error> {
    let endpoint = xwt_web_sys::Endpoint::default();

    let connecting = endpoint.connect(addr).await?;

    let session = connecting.wait_connect().await?;

    Ok(session)
}
