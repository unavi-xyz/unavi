use std::sync::mpsc::Sender;

use bevy::{prelude::*, tasks::futures_lite::StreamExt};
use tarpc::tokio_util::codec::LengthDelimitedCodec;
use tokio::io::AsyncReadExt;
use unavi_server_service::{
    TRANSFORM_LENGTH_FIELD_LENGTH, TRANSFORM_MAX_FRAME_LENGTH, TrackingUpdate,
    from_server::TransformMeta,
};
use wtransport::RecvStream;

use crate::space::Space;

pub struct RecievedTransform {
    player: u64,
}

pub async fn recv_transform_stream(
    mut stream: RecvStream,
    transform_tx: Sender<RecievedTransform>,
) -> anyhow::Result<()> {
    let meta_len = stream.read_u16().await? as usize;

    let mut meta_buf = vec![0; meta_len];
    stream.read_exact(&mut meta_buf).await?;

    let (meta, _) =
        bincode::decode_from_slice::<TransformMeta, _>(&meta_buf, bincode::config::standard())?;

    let mut framed = LengthDelimitedCodec::builder()
        .little_endian()
        .length_field_length(TRANSFORM_LENGTH_FIELD_LENGTH)
        .max_frame_length(TRANSFORM_MAX_FRAME_LENGTH)
        .new_read(stream);

    while let Some(frame) = framed.next().await {
        let bytes = frame?;

        let (_update, _) = bincode::serde::decode_from_slice::<TrackingUpdate, _>(
            &bytes,
            bincode::config::standard(),
        )?;

        transform_tx.send(RecievedTransform {
            player: meta.player,
        })?;
    }

    Ok(())
}

pub fn apply_player_transforms(spaces: Query<&Space>) {
    for space in spaces.iter() {
        let Ok(rx) = space.transform_rx.lock() else {
            continue;
        };

        while let Ok(_update) = rx.try_recv() {
            //
        }
    }
}
