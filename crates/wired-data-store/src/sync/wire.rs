use anyhow::{Context, Result};
use iroh::endpoint::{RecvStream, SendStream};

use super::messages::SyncMessage;

const MAX_MESSAGE_LEN: usize = 16 * 1024 * 1024;

/// Reads a length-prefixed message from a receive stream.
pub async fn read_message(recv: &mut RecvStream) -> Result<SyncMessage> {
    // Read length prefix (4 bytes, big-endian).
    let mut len_buf = [0u8; 4];
    recv.read_exact(&mut len_buf)
        .await
        .context("read message length")?;
    let len = (u32::from_be_bytes(len_buf) as usize).min(MAX_MESSAGE_LEN);

    // Read message body.
    let mut buf = vec![0u8; len];
    recv.read_exact(&mut buf)
        .await
        .context("read message body")?;

    // Decode message.
    let (message, _) =
        bincode::decode_from_slice(&buf, bincode::config::standard()).context("decode message")?;

    Ok(message)
}

/// Writes a length-prefixed message to a send stream.
pub async fn write_message(send: &mut SendStream, message: &SyncMessage) -> Result<()> {
    // Encode message.
    let buf =
        bincode::encode_to_vec(message, bincode::config::standard()).context("encode message")?;

    // Write length prefix (4 bytes, big-endian).
    let len = u32::try_from(buf.len()).context("message too large")?;
    send.write_all(&len.to_be_bytes())
        .await
        .context("write message length")?;

    // Write message body.
    send.write_all(&buf).await.context("write message body")?;

    Ok(())
}
