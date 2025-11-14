use std::{
    collections::{HashMap, HashSet, hash_map::Entry},
    time::Instant,
};

use futures::SinkExt;
use tarpc::tokio_util::codec::{FramedWrite, LengthDelimitedCodec};
use tokio::time::interval;
use tracing::{error, warn};
use unavi_server_service::{
    TRANSFORM_LENGTH_FIELD_LENGTH, TRANSFORM_MAX_FRAME_LENGTH, TrackingIFrame, TrackingPFrame,
    TrackingUpdate, from_server,
};
use wtransport::{Connection, SendStream};

use crate::session::{PlayerId, ServerContext, TICKRATE};

pub async fn handle_pull_transforms(
    ctx: ServerContext,
    player_id: PlayerId,
    connection: Connection,
) -> anyhow::Result<()> {
    let mut subscriptions = HashMap::new();
    let mut last_send_times = HashMap::new();
    let mut pending_updates = HashSet::new();
    let mut player_streams = HashMap::new();
    let mut players_in_shared_spaces = HashSet::new();
    let mut ready_updates = Vec::new();
    let mut check_interval = interval(TICKRATE / 2);

    loop {
        check_interval.tick().await;

        let players_guard = ctx.players.read().await;
        let Some(player) = players_guard.get(&player_id) else {
            return Ok(());
        };

        let spaces_guard = ctx.spaces.read().await;
        players_in_shared_spaces.clear();
        for space_id in &player.spaces {
            if let Some(space) = spaces_guard.get(space_id) {
                for &other_player_id in &space.players {
                    if other_player_id != player_id {
                        players_in_shared_spaces.insert(other_player_id);
                    }
                }
            }
        }
        drop(spaces_guard);

        for &other_player_id in &players_in_shared_spaces {
            if !subscriptions.contains_key(&other_player_id)
                && let Some(other_player) = players_guard.get(&other_player_id)
            {
                let iframe_rx = other_player.iframe_tx.subscribe();
                let pframe_rx = other_player.pframe_tx.subscribe();
                subscriptions.insert(other_player_id, (iframe_rx, pframe_rx));
            }
        }
        drop(players_guard);

        subscriptions.retain(|&other_player_id, _| {
            let keep = players_in_shared_spaces.contains(&other_player_id);
            if !keep {
                last_send_times.remove(&other_player_id);
                pending_updates.remove(&other_player_id);
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

            if can_send_now(&ctx, player_id, other_player_id, &last_send_times).await {
                let iframe = if has_iframe_update {
                    Some(iframe_rx.borrow_and_update().clone())
                } else {
                    None
                };

                let pframe = if has_iframe_update || has_pframe_update {
                    Some(pframe_rx.borrow_and_update().clone())
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
                pending_updates.remove(&other_player_id);
            } else {
                pending_updates.insert(other_player_id);
            }
        }

        ready_updates.clear();
        for &other_player_id in &pending_updates {
            if can_send_now(&ctx, player_id, other_player_id, &last_send_times).await {
                ready_updates.push(other_player_id);
            }
        }

        for &other_player_id in &ready_updates {
            if let Some((iframe_rx, pframe_rx)) = subscriptions.get_mut(&other_player_id) {
                let has_iframe_update = iframe_rx.has_changed().unwrap_or(false);

                let iframe = if has_iframe_update {
                    Some(iframe_rx.borrow_and_update().clone())
                } else {
                    None
                };

                let pframe = Some(pframe_rx.borrow_and_update().clone());

                if let Err(e) = send_updates_to_player(
                    &mut player_streams,
                    &connection,
                    other_player_id,
                    iframe,
                    pframe,
                )
                .await
                {
                    warn!("Failed to send pending transform update: {e}");
                    player_streams.remove(&other_player_id);
                    continue;
                }

                last_send_times.insert(other_player_id, Instant::now());
                pending_updates.remove(&other_player_id);
            }
        }
    }
}

async fn can_send_now(
    ctx: &ServerContext,
    observer_id: PlayerId,
    observed_id: PlayerId,
    last_send_times: &HashMap<PlayerId, Instant>,
) -> bool {
    let tickrates = ctx.player_tickrates.read().await;
    let min_interval = tickrates
        .get(&observer_id)
        .and_then(|map| map.get(&observed_id))
        .copied()
        .unwrap_or(TICKRATE);
    drop(tickrates);

    if let Some(&last_send) = last_send_times.get(&observed_id) {
        last_send.elapsed() >= min_interval
    } else {
        true
    }
}

async fn get_or_create_stream<'a>(
    player_streams: &'a mut HashMap<PlayerId, FramedWrite<SendStream, LengthDelimitedCodec>>,
    connection: &Connection,
    player_id: PlayerId,
) -> anyhow::Result<&'a mut FramedWrite<SendStream, LengthDelimitedCodec>> {
    if let Entry::Vacant(e) = player_streams.entry(player_id) {
        let stream = connection.open_uni().await?.await?;

        let mut framed = LengthDelimitedCodec::builder()
            .little_endian()
            .length_field_length(TRANSFORM_LENGTH_FIELD_LENGTH)
            .max_frame_length(TRANSFORM_MAX_FRAME_LENGTH)
            .new_write(stream);

        let header = from_server::StreamHeader::Transform;
        let header_bytes = bincode::encode_to_vec(&header, bincode::config::standard())?;
        framed.send(header_bytes.into()).await?;

        let meta = from_server::TransformMeta { player: player_id };
        let meta_bytes = bincode::encode_to_vec(&meta, bincode::config::standard())?;
        framed.send(meta_bytes.into()).await?;

        e.insert(framed);
    }

    Ok(player_streams.get_mut(&player_id).unwrap())
}

async fn send_updates_to_player(
    player_streams: &mut HashMap<PlayerId, FramedWrite<SendStream, LengthDelimitedCodec>>,
    connection: &Connection,
    other_player_id: PlayerId,
    iframe: Option<TrackingIFrame>,
    pframe: Option<TrackingPFrame>,
) -> anyhow::Result<()> {
    let framed = get_or_create_stream(player_streams, connection, other_player_id).await?;

    if let Some(iframe) = iframe {
        let iframe_update = TrackingUpdate::IFrame(iframe);
        let iframe_bytes =
            bincode::serde::encode_to_vec(&iframe_update, bincode::config::standard())?;
        framed.send(iframe_bytes.into()).await?;
    }

    if let Some(pframe) = pframe {
        let pframe_update = TrackingUpdate::PFrame(pframe);
        let pframe_bytes =
            bincode::serde::encode_to_vec(&pframe_update, bincode::config::standard())?;
        framed.send(pframe_bytes.into()).await?;
    }

    Ok(())
}
