use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use tracing::info;
use unavi_server_service::from_client::StreamHeader;
use wtransport::RecvStream;

use crate::session::ServerContext;

pub mod control;
pub mod transform;
mod voice;

#[derive(Default, Clone)]
pub struct StreamCounts {
    transform: Arc<AtomicUsize>,
    voice: Arc<AtomicUsize>,
}

pub async fn recv_stream(
    ctx: ServerContext,
    counts: StreamCounts,
    player_id: u64,
    mut stream: RecvStream,
) -> anyhow::Result<()> {
    let mut header_buf = [0u8; 1];
    stream.read_exact(&mut header_buf).await?;
    let (header, _) = bincode::decode_from_slice(&header_buf, bincode::config::standard())?;

    match header {
        StreamHeader::TransformIFrame => {
            let count = counts.transform.fetch_add(1, Ordering::SeqCst);

            if count == 0 {
                let res = transform::recv_iframe_stream(ctx, player_id, stream).await;
                counts.transform.fetch_sub(1, Ordering::SeqCst);
                res?;
            } else {
                info!("ignoring new stream, i-frame stream already established");
                counts.transform.fetch_sub(1, Ordering::SeqCst);
            }
        }
        StreamHeader::TransformPFrame => {
            transform::recv_pframe_stream(ctx, player_id, stream).await?;
        }
        StreamHeader::Voice => {
            let count = counts.voice.fetch_add(1, Ordering::SeqCst);

            if count == 0 {
                let res = voice::recv_voice_stream(stream);
                counts.voice.fetch_sub(1, Ordering::SeqCst);
                res?;
            } else {
                info!("ignoring new stream, voice stream already established");
                counts.voice.fetch_sub(1, Ordering::SeqCst);
            }
        }
    }

    Ok(())
}
