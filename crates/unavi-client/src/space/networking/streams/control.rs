use bevy::prelude::*;
use futures::StreamExt;
use tarpc::tokio_util::codec::LengthDelimitedCodec;
use unavi_server_service::from_server::ControlMessage;
use wtransport::RecvStream;

use crate::space::{Host, HostControlChannel, HostTransformChannels, PlayerHost, RemotePlayer};

pub async fn recv_control_stream(
    stream: RecvStream,
    control_tx: flume::Sender<ControlMessage>,
) -> anyhow::Result<()> {
    let mut framed = LengthDelimitedCodec::builder()
        .little_endian()
        .length_field_length(2)
        .max_frame_length(512)
        .new_read(stream);

    while let Some(bytes) = framed.next().await {
        let (msg, _) =
            bincode::decode_from_slice::<ControlMessage, _>(&bytes?, bincode::config::standard())?;

        if let Err(e) = control_tx.try_send(msg) {
            warn!("Dropped control message: {:?}", e);
        }
    }

    Ok(())
}

pub fn apply_controls(
    mut commands: Commands,
    hosts: Query<(Entity, &Host, &HostControlChannel, &HostTransformChannels)>,
    remote_players: Query<(Entity, &RemotePlayer, &PlayerHost)>,
) {
    for (host_entity, _, channel, transform_channels) in hosts.iter() {
        // Drain all pending messages.
        let mut messages = Vec::new();
        while let Ok(msg) = channel.rx.try_recv() {
            messages.push(msg);
        }

        // Process all messages in order.
        for msg in messages {
            match msg {
                ControlMessage::PlayerLeft { player_id } => {
                    info!("Player {player_id}@{host_entity} left");

                    // Remove from transform channels to prevent re-spawn.
                    let _ = transform_channels.players.remove_sync(&player_id);

                    for (entity, player, player_host) in remote_players.iter() {
                        if player_host.0 == host_entity && player.player_id == player_id {
                            commands.entity(entity).despawn();
                        }
                    }
                }
            }
        }
    }
}
