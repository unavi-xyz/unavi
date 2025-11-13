use bevy::prelude::*;
use tokio::io::AsyncReadExt;
use unavi_server_service::from_server::StreamHeader;
use wtransport::RecvStream;

pub mod publish;
mod transform;
mod voice;

pub async fn recv_stream(mut stream: RecvStream) -> anyhow::Result<()> {
    let header_len = stream.read_u16().await? as usize;

    let mut header_buf = vec![0; header_len];
    stream.read_exact(&mut header_buf).await?;

    let (header, _) = bincode::decode_from_slice(&header_buf, bincode::config::standard())?;

    match header {
        StreamHeader::Transform => {
            transform::recv_transform_stream(stream).await?;
        }
        StreamHeader::Voice => {
            voice::recv_voice_stream(stream).await?;
        }
    }

    Ok(())
}
