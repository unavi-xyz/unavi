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
    let mut stream = connection.open_uni().await?.await?;

    // Send stream header.
    let header = StreamHeader::Control;
    let header_bytes = bincode::encode_to_vec(&header, bincode::config::standard())?;
    stream.write_all(&header_bytes).await?;

    let mut framed = LengthDelimitedCodec::builder()
        .little_endian()
        .length_field_length(2)
        .max_frame_length(512)
        .new_write(stream);

    let mut space_subscriptions: HashMap<SpaceId, Receiver<ControlMessage>> = HashMap::new();
    let mut check_interval = interval(TICKRATE);

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

        // Subscribe to control broadcasts for all spaces player is in.
        for space_id in &player_spaces {
            if !space_subscriptions.contains_key(space_id) {
                let rx = ctx
                    .spaces
                    .read_async(space_id, |_, space| space.control_tx.subscribe())
                    .await;

                if let Some(rx) = rx {
                    space_subscriptions.insert(space_id.clone(), rx);
                }
            }
        }

        // Clean up subscriptions for spaces player left.
        space_subscriptions.retain(|space_id, _| player_spaces.contains(space_id));

        // Process any pending control messages from all subscriptions.
        for rx in space_subscriptions.values_mut() {
            loop {
                match rx.try_recv() {
                    Ok(msg) => {
                        let msg_bytes = bincode::encode_to_vec(&msg, bincode::config::standard())?;
                        framed.send(msg_bytes.into()).await?;
                    }
                    Err(
                        tokio::sync::broadcast::error::TryRecvError::Empty
                        | tokio::sync::broadcast::error::TryRecvError::Closed,
                    ) => {
                        break;
                    }
                    Err(tokio::sync::broadcast::error::TryRecvError::Lagged(n)) => {
                        error!("Control stream lagged by {n} messages");
                        break;
                    }
                }
            }
        }
    }
}
