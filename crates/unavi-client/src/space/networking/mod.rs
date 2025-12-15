use std::sync::Arc;

use bevy::prelude::*;

pub mod connection;
pub mod streams;
pub mod thread;

use thread::{NetworkEvent, NetworkingThread};

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        let networking_thread = NetworkingThread::spawn();

        app.insert_resource(networking_thread)
            .add_observer(connection::cleanup::handle_space_despawn)
            .add_systems(
                FixedUpdate,
                (
                    handle_network_events,
                    connection::lifecycle::drive_connection_lifecycle,
                    streams::control::apply_controls,
                    streams::publish::publish_transform_data,
                    streams::transform::apply_player_transforms,
                ),
            )
            .add_systems(Update, streams::transform::smooth_network_transforms)
            .add_systems(Last, connection::cleanup::cleanup_connections_on_exit);
    }
}

fn handle_network_events(
    networking: Res<NetworkingThread>,
    mut commands: Commands,
    mut spaces: Query<&mut connection::state::ConnectionAttempt>,
    hosts: Query<(
        Entity,
        &crate::space::Host,
        Option<&crate::space::HostPlayers>,
    )>,
) {
    use crate::space::{Host, HostControlChannel, HostTransformChannels};
    use connection::state::ConnectionState;

    while let Ok(event) = networking.event_rx.try_recv() {
        match event {
            NetworkEvent::Connected {
                entity,
                host,
                connect_url,
            } => {
                info!("Entity {entity:?} connected to {connect_url}");

                // Update connection state to Connected.
                commands.entity(entity).insert(ConnectionState::Connected);

                // Reset backoff timer.
                if let Ok(mut attempt) = spaces.get_mut(entity) {
                    attempt.reset();
                }

                // Spawn Host entity for this connection.
                commands.spawn((
                    Host {
                        connect_url: connect_url.clone(),
                    },
                    HostTransformChannels {
                        players: Arc::clone(&host.transform_channels),
                    },
                    HostControlChannel {
                        _tx: host.control_tx.clone(),
                        rx: host.control_rx.clone(),
                    },
                ));
            }
            NetworkEvent::ConnectionFailed { entity, attempt } => {
                warn!("Entity {entity:?} connection failed, attempt {attempt}");

                // Update to Reconnecting state.
                commands
                    .entity(entity)
                    .insert(ConnectionState::Reconnecting {
                        attempt: attempt + 1,
                    });

                // Increase backoff.
                if let Ok(mut attempt_comp) = spaces.get_mut(entity) {
                    attempt_comp.increase_backoff();
                }
            }
            NetworkEvent::ConnectionClosed { connect_url } => {
                info!("Connection to {connect_url} closed");

                // Find and cleanup the Host entity.
                for (entity, host, host_players) in hosts.iter() {
                    if host.connect_url == connect_url {
                        // Cleanup any remote players associated with this host.
                        if let Some(players) = host_players {
                            for player in &players.0 {
                                commands.entity(*player).despawn();
                            }
                        }

                        // Despawn the host entity.
                        commands.entity(entity).despawn();
                        break;
                    }
                }
            }
        }
    }
}
