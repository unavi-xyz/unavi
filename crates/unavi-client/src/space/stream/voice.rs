use wtransport::RecvStream;

pub async fn recv_voice_stream(_stream: RecvStream) -> anyhow::Result<()> {
    Ok(())
}
