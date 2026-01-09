use iroh::endpoint::{RecvStream, SendStream};

pub async fn handle_inbound(_tx: SendStream, _rx: RecvStream) -> anyhow::Result<()> {
    Ok(())
}
