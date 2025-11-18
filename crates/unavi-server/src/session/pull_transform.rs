use std::{
    collections::{HashMap, HashSet, hash_map::Entry},
    time::Instant,
};

use futures::{FutureExt, SinkExt};
use smallvec::SmallVec;
use tarpc::tokio_util::codec::{FramedWrite, LengthDelimitedCodec};
use tokio::time::interval;
use tracing::error;
use unavi_server_service::{
    TRANSFORM_LENGTH_FIELD_LENGTH, TRANSFORM_MAX_FRAME_LENGTH, TrackingIFrame, TrackingPFrame,
    from_server,
};
use wtransport::{Connection, SendStream};

use crate::session::{PlayerId, ServerContext, TICKRATE};

struct PlayerStreams {
    iframe_stream: FramedWrite<SendStream, LengthDelimitedCodec>,
    pframe_stream: Option<SendStream>,
}

pub async fn handle_pull_transforms(
    ctx: ServerContext,
    player_id: PlayerId,
    connection: Connection,
) -> anyhow::Result<()> {
    let mut check_interval = interval(TICKRATE / 2);
    let mut last_send_times = HashMap::new();
    let mut player_streams = HashMap::new();
    let mut players_in_shared_spaces = HashSet::new();
    let mut subscriptions = HashMap::new();

    loop {
        check_interval.tick().await;

        let player_spaces = ctx
            .players
            .read_async(&player_id, |_, player| player.spaces.clone())
            .await;

        let Some(player_spaces) = player_spaces else {
            // Exit once the player is disconnected.
            return Ok(());
        };

        players_in_shared_spaces.clear();
        for space_id in &player_spaces {
            let _ = ctx
                .spaces
                .read_async(space_id, |_, space| {
                    for &other_player_id in &space.players {
                        if other_player_id != player_id {
                            players_in_shared_spaces.insert(other_player_id);
                        }
                    }
                })
                .await;
        }

        // Collect new subscriptions and their initial iframes.
        let mut new_subscriptions: SmallVec<[(PlayerId, _, _, TrackingIFrame); 4]> =
            SmallVec::new();
        for &other_player_id in &players_in_shared_spaces {
            if !subscriptions.contains_key(&other_player_id) {
                let player_state = ctx
                    .players
                    .read_async(&other_player_id, |_, other_player| {
                        let iframe_rx = other_player.iframe_tx.subscribe();
                        let pframe_rx = other_player.pframe_tx.subscribe();
                        let initial_iframe = iframe_rx.borrow().clone();
                        (iframe_rx, pframe_rx, initial_iframe)
                    })
                    .await;

                if let Some((iframe_rx, pframe_rx, initial_iframe)) = player_state {
                    new_subscriptions.push((other_player_id, iframe_rx, pframe_rx, initial_iframe));
                }
            }
        }

        // Send initial iframes for new subscriptions.
        for (other_player_id, iframe_rx, pframe_rx, initial_iframe) in new_subscriptions {
            subscriptions.insert(other_player_id, (iframe_rx, pframe_rx));

            if let Err(e) = send_updates_to_player(
                &mut player_streams,
                &connection,
                other_player_id,
                Some(initial_iframe),
                None,
            )
            .await
            {
                error!("Failed to send initial iframe: {e}");
                player_streams.remove(&other_player_id);
                subscriptions.remove(&other_player_id);
                continue;
            }

            last_send_times.insert(other_player_id, Instant::now());
        }

        subscriptions.retain(|&other_player_id, _| {
            let keep = players_in_shared_spaces.contains(&other_player_id);
            if !keep {
                last_send_times.remove(&other_player_id);
                player_streams.remove(&other_player_id);
            }
            keep
        });

        for (&other_player_id, (iframe_rx, pframe_rx)) in &mut subscriptions {
            let has_iframe_update = iframe_rx.has_changed().unwrap_or(false);
            let has_pframe_update = pframe_rx.has_changed().unwrap_or(false);

            if !has_iframe_update && !has_pframe_update {
                continue;
            }

            if !tickrate_ready(&ctx, player_id, other_player_id, &last_send_times).await {
                continue;
            }

            let iframe = if has_iframe_update {
                Some(iframe_rx.borrow_and_update().clone())
            } else {
                None
            };

            let pframe = if has_pframe_update {
                let pframe_update = pframe_rx.borrow_and_update();
                if pframe_update.iframe_id == iframe_rx.borrow().iframe_id {
                    Some(pframe_update.clone())
                } else {
                    None
                }
            } else {
                None
            };

            if let Err(e) = send_updates_to_player(
                &mut player_streams,
                &connection,
                other_player_id,
                iframe,
                pframe,
            )
            .await
            {
                error!("Failed to send transform update: {e}");
                player_streams.remove(&other_player_id);
                continue;
            }

            last_send_times.insert(other_player_id, Instant::now());
        }
    }
}

async fn tickrate_ready(
    ctx: &ServerContext,
    observer_id: PlayerId,
    observed_id: PlayerId,
    last_send_times: &HashMap<PlayerId, Instant>,
) -> bool {
    // Get the tickrate for this observer-observed pair, if it exists.
    // We need to avoid nested futures that borrow, so we collect synchronously.
    let min_interval = ctx
        .player_tickrates
        .get_async(&observer_id)
        .await
        .and_then(|rates_entry| {
            let rates = rates_entry.get();
            rates
                .get_async(&observed_id)
                .now_or_never()?
                .map(|entry| *entry.get())
        })
        .unwrap_or(TICKRATE);

    if let Some(&last_send) = last_send_times.get(&observed_id) {
        last_send.elapsed() >= min_interval
    } else {
        true
    }
}

async fn get_or_create_iframe_stream<'a>(
    player_streams: &'a mut HashMap<PlayerId, PlayerStreams>,
    connection: &Connection,
    player_id: PlayerId,
) -> anyhow::Result<&'a mut PlayerStreams> {
    if let Entry::Vacant(e) = player_streams.entry(player_id) {
        let mut stream = connection.open_uni().await?.await?;

        let header = from_server::StreamHeader::TransformIFrame;
        let header_bytes = bincode::encode_to_vec(&header, bincode::config::standard())?;
        stream.write_all(&header_bytes).await?;

        let mut framed = LengthDelimitedCodec::builder()
            .little_endian()
            .length_field_length(TRANSFORM_LENGTH_FIELD_LENGTH)
            .max_frame_length(TRANSFORM_MAX_FRAME_LENGTH)
            .new_write(stream);

        let meta = from_server::TransformMeta { player: player_id };
        let meta_bytes = bincode::encode_to_vec(&meta, bincode::config::standard())?;
        framed.send(meta_bytes.into()).await?;

        e.insert(PlayerStreams {
            iframe_stream: framed,
            pframe_stream: None,
        });
    }

    Ok(player_streams.get_mut(&player_id).unwrap())
}

async fn send_updates_to_player(
    player_streams: &mut HashMap<PlayerId, PlayerStreams>,
    connection: &Connection,
    other_player_id: PlayerId,
    iframe: Option<TrackingIFrame>,
    pframe: Option<TrackingPFrame>,
) -> anyhow::Result<()> {
    let streams = get_or_create_iframe_stream(player_streams, connection, other_player_id).await?;

    if let Some(iframe) = iframe {
        let iframe_bytes = bincode::serde::encode_to_vec(&iframe, bincode::config::standard())?;
        streams.iframe_stream.send(iframe_bytes.into()).await?;
    }

    if let Some(pframe) = pframe {
        if let Some(mut old_stream) = streams.pframe_stream.take() {
            let _ = old_stream.reset(0u32.into());
        }

        let mut stream = connection.open_uni().await?.await?;
        stream.set_priority(10);

        let header = from_server::StreamHeader::TransformPFrame;
        let header_bytes = bincode::encode_to_vec(&header, bincode::config::standard())?;
        stream.write_all(&header_bytes).await?;

        let mut framed = LengthDelimitedCodec::builder()
            .little_endian()
            .length_field_length(TRANSFORM_LENGTH_FIELD_LENGTH)
            .max_frame_length(TRANSFORM_MAX_FRAME_LENGTH)
            .new_write(stream);

        let meta = from_server::TransformMeta {
            player: other_player_id,
        };
        let meta_bytes = bincode::encode_to_vec(&meta, bincode::config::standard())?;
        framed.send(meta_bytes.into()).await?;

        let pframe_bytes = bincode::serde::encode_to_vec(&pframe, bincode::config::standard())?;
        framed.send(pframe_bytes.into()).await?;

        let send_stream = framed.into_inner();
        streams.pframe_stream = Some(send_stream);
    }

    Ok(())
}
