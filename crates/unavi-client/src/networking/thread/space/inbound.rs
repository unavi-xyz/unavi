use iroh::endpoint::RecvStream;

pub async fn handle_inbound(_rx: RecvStream) -> anyhow::Result<()> {
    Ok(())
}
