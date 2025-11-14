use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use tokio::io::AsyncReadExt;
use unavi_server_service::from_client::StreamHeader;
use wtransport::RecvStream;

use crate::session::ServerContext;

pub mod transform;
mod voice;

#[derive(Default, Clone)]
pub struct StreamCounts {
    transform: Arc<AtomicUsize>,
    voice: Arc<AtomicUsize>,
}

pub async fn handle_stream(
    ctx: ServerContext,
    counts: StreamCounts,
    player_id: u64,
    mut stream: RecvStream,
) -> anyhow::Result<()> {
    let header_len = stream.read_u16_le().await? as usize;
    let mut header_buf = vec![0; header_len];
    stream.read_exact(&mut header_buf).await?;

    let (header, _) = bincode::decode_from_slice(&header_buf, bincode::config::standard())?;

    match header {
        StreamHeader::Transform => {
            let count = counts.transform.fetch_add(1, Ordering::SeqCst);

            if count == 0 {
                let res = transform::handle_transform_stream(ctx, player_id, stream).await;
                counts.transform.fetch_sub(1, Ordering::SeqCst);
                res?;
            } else {
                counts.transform.fetch_sub(1, Ordering::SeqCst);
            }
        }
        StreamHeader::Voice => {
            let count = counts.voice.fetch_add(1, Ordering::SeqCst);

            if count == 0 {
                let res = voice::handle_voice_stream(stream).await;
                counts.voice.fetch_sub(1, Ordering::SeqCst);
                res?;
            } else {
                counts.voice.fetch_sub(1, Ordering::SeqCst);
            }
        }
    }

    Ok(())
}
