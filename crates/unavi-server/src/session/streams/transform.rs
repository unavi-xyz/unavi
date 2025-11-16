use futures::StreamExt;
use tarpc::tokio_util::codec::LengthDelimitedCodec;
use unavi_server_service::{
    TRANSFORM_LENGTH_FIELD_LENGTH, TRANSFORM_MAX_FRAME_LENGTH, TrackingPFrame, TrackingUpdate,
    from_client::TransformMeta,
};
use wtransport::RecvStream;

use crate::session::ServerContext;

pub async fn handle_transform_stream(
    ctx: ServerContext,
    player_id: u64,
    stream: RecvStream,
) -> anyhow::Result<()> {
    let mut framed = LengthDelimitedCodec::builder()
        .little_endian()
        .length_field_length(TRANSFORM_LENGTH_FIELD_LENGTH)
        .max_frame_length(TRANSFORM_MAX_FRAME_LENGTH)
        .new_read(stream);

    let Some(meta_bytes) = framed.next().await else {
        return Ok(());
    };

    let (_meta, _) =
        bincode::decode_from_slice::<TransformMeta, _>(&meta_bytes?, bincode::config::standard())?;

    while let Some(bytes) = framed.next().await {
        let (update, _) = bincode::serde::decode_from_slice::<TrackingUpdate, _>(
            &bytes?,
            bincode::config::standard(),
        )?;

        let players = ctx.players.read().await;
        let Some(player) = players.get(&player_id) else {
            return Ok(());
        };

        match update {
            TrackingUpdate::IFrame(iframe) => {
                let _ = player.iframe_tx.send(iframe);
                let _ = player.pframe_tx.send(TrackingPFrame::default());
            }
            TrackingUpdate::PFrame(pframe) => {
                let _ = player.pframe_tx.send(pframe);
            }
        }
    }

    Ok(())
}
