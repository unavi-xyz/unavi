use std::collections::HashMap;

use futures::SinkExt;
use tarpc::tokio_util::codec::LengthDelimitedCodec;
use tokio::{sync::broadcast::Receiver, time::interval};
use tracing::error;
use unavi_server_service::from_server::{ControlMessage, StreamHeader};
use wtransport::Connection;

use crate::session::{PlayerId, ServerContext, SpaceId, TICKRATE};

pub async fn handle_control_stream(
    ctx: ServerContext,
    player_id: PlayerId,
    connection: Connection,
) -> anyhow::Result<()> {
    let stream = connection.open_uni().await?.await?;

    let mut framed = LengthDelimitedCodec::builder()
        .little_endian()
        .length_field_length(2)
        .new_write(stream);

    // Send stream header.
    let header = StreamHeader::Control;
    let header_bytes = bincode::encode_to_vec(&header, bincode::config::standard())?;
    framed.send(header_bytes.into()).await?;

    let mut space_subscriptions: HashMap<SpaceId, Receiver<ControlMessage>> = HashMap::new();
    let mut check_interval = interval(TICKRATE / 2);

    loop {
        check_interval.tick().await;

        let players_guard = ctx.players.read().await;
        let Some(player) = players_guard.get(&player_id) else {
            return Ok(());
        };

        let spaces_guard = ctx.spaces.read().await;

        // Subscribe to control broadcasts for all spaces player is in.
        for space_id in &player.spaces {
            if !space_subscriptions.contains_key(space_id)
                && let Some(space) = spaces_guard.get(space_id)
            {
                let rx = space.control_tx.subscribe();
                space_subscriptions.insert(space_id.clone(), rx);
            }
        }

        // Clean up subscriptions for spaces player left.
        space_subscriptions.retain(|space_id, _| player.spaces.contains(space_id));

        drop(players_guard);
        drop(spaces_guard);

        // Process any pending control messages from all subscriptions.
        for rx in space_subscriptions.values_mut() {
            loop {
                match rx.try_recv() {
                    Ok(msg) => {
                        let msg_bytes = bincode::encode_to_vec(&msg, bincode::config::standard())?;
                        framed.send(msg_bytes.into()).await?;
                    }
                    Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {
                        break;
                    }
                    Err(tokio::sync::broadcast::error::TryRecvError::Lagged(n)) => {
                        error!("Control stream lagged by {n} messages");
                        break;
                    }
                    Err(tokio::sync::broadcast::error::TryRecvError::Closed) => {
                        break;
                    }
                }
            }
        }
    }
}
