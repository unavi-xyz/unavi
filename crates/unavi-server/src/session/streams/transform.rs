use futures::StreamExt;
use tarpc::tokio_util::codec::LengthDelimitedCodec;
use unavi_server_service::{
    TRANSFORM_LENGTH_FIELD_LENGTH, TRANSFORM_MAX_FRAME_LENGTH, TrackingIFrame, TrackingPFrame,
};
use wtransport::RecvStream;

use crate::session::ServerContext;

pub async fn recv_iframe_stream(
    ctx: ServerContext,
    player_id: u64,
    stream: RecvStream,
) -> anyhow::Result<()> {
    let mut framed = LengthDelimitedCodec::builder()
        .little_endian()
        .length_field_length(TRANSFORM_LENGTH_FIELD_LENGTH)
        .max_frame_length(TRANSFORM_MAX_FRAME_LENGTH)
        .new_read(stream);

    while let Some(bytes) = framed.next().await {
        let (iframe, _) = bincode::serde::decode_from_slice::<TrackingIFrame, _>(
            &bytes?,
            bincode::config::standard(),
        )?;

        let tx = ctx
            .players
            .read_async(&player_id, |_, player| player.iframe_tx.clone())
            .await;

        let Some(tx) = tx else {
            return Ok(());
        };

        tx.send_modify(|prev| {
            prev.iframe_id = iframe.iframe_id;
            prev.translation = iframe.translation;
            prev.rotation = iframe.rotation;

            for joint in iframe.joints {
                if let Some(found) = prev.joints.iter_mut().find(|j| j.id == joint.id) {
                    found.rotation = joint.rotation;
                } else {
                    prev.joints.push(joint);
                }
            }
        });
    }

    Ok(())
}

pub async fn recv_pframe_stream(
    ctx: ServerContext,
    player_id: u64,
    stream: RecvStream,
) -> anyhow::Result<()> {
    let mut framed = LengthDelimitedCodec::builder()
        .little_endian()
        .length_field_length(TRANSFORM_LENGTH_FIELD_LENGTH)
        .max_frame_length(TRANSFORM_MAX_FRAME_LENGTH)
        .new_read(stream);

    if let Some(bytes) = framed.next().await {
        let (pframe, _) = bincode::serde::decode_from_slice::<TrackingPFrame, _>(
            &bytes?,
            bincode::config::standard(),
        )?;

        let tx = ctx
            .players
            .read_async(&player_id, |_, player| player.pframe_tx.clone())
            .await;

        let Some(tx) = tx else {
            return Ok(());
        };

        let _ = tx.send(pframe);
    }

    Ok(())
}
