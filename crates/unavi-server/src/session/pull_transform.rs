use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

use futures::SinkExt;
use tarpc::tokio_util::codec::LengthDelimitedCodec;
use tokio::{io::AsyncWriteExt, sync::watch, time::interval};
use tracing::{error, warn};
use unavi_server_service::{
    TRANSFORM_LENGTH_FIELD_LENGTH, TRANSFORM_MAX_FRAME_LENGTH, TrackingIFrame, TrackingPFrame,
    TrackingUpdate, from_server,
};
use wtransport::Connection;

use crate::session::{PlayerId, ServerContext, TICKRATE};

pub async fn handle_pull_transforms(
    ctx: ServerContext,
    player_id: PlayerId,
    connection: Connection,
) -> anyhow::Result<()> {
    let mut subscriptions: HashMap<
        PlayerId,
        (
            watch::Receiver<(usize, TrackingIFrame)>,
            watch::Receiver<TrackingPFrame>,
        ),
    > = HashMap::new();
    let mut last_send_times: HashMap<PlayerId, Instant> = HashMap::new();
    let mut pending_updates: HashSet<PlayerId> = HashSet::new();
    let mut check_interval = interval(TICKRATE / 2);

    loop {
        check_interval.tick().await;

        let players_guard = ctx.players.read().await;
        let Some(player) = players_guard.get(&player_id) else {
            return Ok(());
        };
        let player_spaces = player.spaces.clone();
        drop(players_guard);

        let spaces_guard = ctx.spaces.read().await;
        let mut players_in_shared_spaces = HashSet::new();
        for space_id in &player_spaces {
            if let Some(space) = spaces_guard.get(space_id) {
                for &other_player_id in &space.players {
                    if other_player_id != player_id {
                        players_in_shared_spaces.insert(other_player_id);
                    }
                }
            }
        }
        drop(spaces_guard);

        let players_guard = ctx.players.read().await;
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

        subscriptions
            .retain(|&other_player_id, _| players_in_shared_spaces.contains(&other_player_id));
        last_send_times
            .retain(|&other_player_id, _| players_in_shared_spaces.contains(&other_player_id));
        pending_updates
            .retain(|&other_player_id| players_in_shared_spaces.contains(&other_player_id));

        for (&other_player_id, (iframe_rx, pframe_rx)) in &mut subscriptions {
            let has_iframe_update = iframe_rx.has_changed().unwrap_or(false);
            let has_pframe_update = pframe_rx.has_changed().unwrap_or(false);

            if !has_iframe_update && !has_pframe_update {
                continue;
            }

            if can_send_now(&ctx, player_id, other_player_id, &last_send_times).await {
                let _ = iframe_rx.borrow_and_update();
                let _ = pframe_rx.borrow_and_update();

                let iframe = iframe_rx.borrow().clone();
                let pframe = pframe_rx.borrow().clone();

                if let Err(e) =
                    send_transform_update(&connection, other_player_id, iframe.1, pframe).await
                {
                    error!("Failed to send transform update: {e}");
                    continue;
                }

                last_send_times.insert(other_player_id, Instant::now());
                pending_updates.remove(&other_player_id);
            } else {
                pending_updates.insert(other_player_id);
            }
        }

        let mut ready_updates = Vec::new();
        for &other_player_id in &pending_updates {
            if can_send_now(&ctx, player_id, other_player_id, &last_send_times).await {
                ready_updates.push(other_player_id);
            }
        }

        for other_player_id in ready_updates {
            if let Some((iframe_rx, pframe_rx)) = subscriptions.get_mut(&other_player_id) {
                let _ = iframe_rx.borrow_and_update();
                let _ = pframe_rx.borrow_and_update();

                let iframe = iframe_rx.borrow().clone();
                let pframe = pframe_rx.borrow().clone();

                if let Err(e) =
                    send_transform_update(&connection, other_player_id, iframe.1, pframe).await
                {
                    warn!("Failed to send pending transform update: {e}");
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

async fn send_transform_update(
    connection: &Connection,
    player_id: PlayerId,
    iframe: TrackingIFrame,
    pframe: TrackingPFrame,
) -> anyhow::Result<()> {
    let mut stream = connection.open_uni().await?.await?;

    let header = from_server::StreamHeader::Transform;
    let header_bytes = bincode::encode_to_vec(&header, bincode::config::standard())?;
    stream.write_u16(header_bytes.len() as u16).await?;
    stream.write_all(&header_bytes).await?;

    let meta = from_server::TransformMeta { player: player_id };
    let meta_bytes = bincode::encode_to_vec(&meta, bincode::config::standard())?;
    stream.write_u16(meta_bytes.len() as u16).await?;
    stream.write_all(&meta_bytes).await?;

    let mut framed = LengthDelimitedCodec::builder()
        .little_endian()
        .length_field_length(TRANSFORM_LENGTH_FIELD_LENGTH)
        .max_frame_length(TRANSFORM_MAX_FRAME_LENGTH)
        .new_write(stream);

    let iframe_update = TrackingUpdate::IFrame(iframe);
    let iframe_bytes = bincode::serde::encode_to_vec(&iframe_update, bincode::config::standard())?;
    framed.send(iframe_bytes.into()).await?;

    let pframe_update = TrackingUpdate::PFrame(pframe);
    let pframe_bytes = bincode::serde::encode_to_vec(&pframe_update, bincode::config::standard())?;
    framed.send(pframe_bytes.into()).await?;

    Ok(())
}
