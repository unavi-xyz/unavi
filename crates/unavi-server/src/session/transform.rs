use futures::StreamExt;
use tarpc::tokio_util::codec::LengthDelimitedCodec;
use tokio::{
    io::AsyncReadExt,
    time::{MissedTickBehavior, interval},
};
use tracing::{info, warn};
use unavi_server_service::{TrackingPFrame, TrackingUpdate, from_client::TransformMeta};
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

    let (meta, _) =
        bincode::decode_from_slice::<TransformMeta, _>(&meta_buf, bincode::config::standard())?;

    info!("Got transform stream: {meta:?}");

    let mut framed = LengthDelimitedCodec::builder()
        .little_endian()
        .length_field_length(2)
        .max_frame_length(512)
        .new_read(stream);

    let mut interval = interval(TICKRATE);
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    while let Some(frame) = framed.next().await {
        let bytes = frame?;

        let (update, _) = bincode::serde::decode_from_slice::<TrackingUpdate, _>(
            &bytes,
            bincode::config::standard(),
        )?;

        match update {
            TrackingUpdate::IFrame(_) => {
                // I-frame data is pushed.
                // TODO: make pull
                // Send to all space players.
                let players = ctx.players.read().await;
                let Some(player) = players.get(&player_id) else {
                    return Ok(());
                };

                let spaces = ctx.spaces.read().await;

                for space_id in player.spaces.iter() {
                    let Some(space) = spaces.get(space_id) else {
                        warn!("space not found");
                        continue;
                    };

                    for other_player_id in space.players.iter() {
                        if *other_player_id == player_id {
                            continue;
                        };
                        let Some(other_player) = players.get(other_player_id) else {
                            continue;
                        };

                        let other_con = other_player.connection.clone();
                        let bytes = bytes.clone();

                        tokio::spawn(async move {
                            match other_con.open_uni().await {
                                Ok(opening) => {
                                    let mut other_out = opening.await?;

                                    let out_meta =
                                        unavi_server_service::from_server::TransformMeta {
                                            player: player_id,
                                        };
                                    let out_meta_vec = bincode::encode_to_vec(
                                        out_meta,
                                        bincode::config::standard(),
                                    )?;

                                    other_out.write_all(&out_meta_vec).await?;
                                    other_out.write_all(&bytes).await?;
                                }
                                Err(e) => {
                                    warn!("Failed to open i-frame stream: {e:?}");
                                }
                            }

                            Ok::<_, anyhow::Error>(())
                        });
                    }
                }

                // Reset p-frame.
                let mut players = ctx.players.write().await;
                let Some(player) = players.get_mut(&player_id) else {
                    return Ok(());
                };
                player.latest_pframe = TrackingPFrame::default();
            }
            TrackingUpdate::PFrame(msg) => {
                // P-frame data is pulled.
                // Save in player data.
                let mut players = ctx.players.write().await;
                let Some(player) = players.get_mut(&player_id) else {
                    return Ok(());
                };
                player.latest_pframe = msg;
            }
        }

        interval.tick().await;
    }

    Ok(())
}
