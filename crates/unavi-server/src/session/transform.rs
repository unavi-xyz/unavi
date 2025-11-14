use std::time::Duration;

use futures::StreamExt;
use tarpc::tokio_util::codec::LengthDelimitedCodec;
use tokio::{io::AsyncReadExt, time::sleep};
use unavi_server_service::{
    TRANSFORM_LENGTH_FIELD_LENGTH, TRANSFORM_MAX_FRAME_LENGTH, TrackingPFrame, TrackingUpdate,
    from_client::TransformMeta,
};
use wtransport::RecvStream;

use crate::session::{ServerContext, TICKRATE};

pub async fn handle_transform_stream(
    ctx: ServerContext,
    player_id: u64,
    mut stream: RecvStream,
) -> anyhow::Result<()> {
    let meta_len = stream.read_u16().await? as usize;

    let mut meta_buf = vec![0; meta_len];
    stream.read_exact(&mut meta_buf).await?;

    let (_meta, _) =
        bincode::decode_from_slice::<TransformMeta, _>(&meta_buf, bincode::config::standard())?;

    let mut framed = LengthDelimitedCodec::builder()
        .little_endian()
        .length_field_length(TRANSFORM_LENGTH_FIELD_LENGTH)
        .max_frame_length(TRANSFORM_MAX_FRAME_LENGTH)
        .new_read(stream);

    let mut iframe_count = 0;

    while let Some(frame) = framed.next().await {
        let bytes = frame?;

        let (update, _) = bincode::serde::decode_from_slice::<TrackingUpdate, _>(
            &bytes,
            bincode::config::standard(),
        )?;

        let players = ctx.players.read().await;
        let Some(player) = players.get(&player_id) else {
            return Ok(());
        };

        match update {
            TrackingUpdate::IFrame(iframe) => {
                iframe_count += 1;
                let _ = player.iframe_tx.send((iframe_count, iframe));
                let _ = player.pframe_tx.send(TrackingPFrame::default());
            }
            TrackingUpdate::PFrame(pframe) => {
                let _ = player.pframe_tx.send(pframe);
            }
        }

        drop(players);

        sleep(Duration::from_millis(TICKRATE.as_millis() as u64 / 4)).await;
    }

    Ok(())
}
