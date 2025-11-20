use bevy::{log::*, prelude::*};

use super::state::{ConnectionAttempt, ConnectionState};
use crate::space::{
    Space,
    networking::{NetworkCommand, NetworkingThread},
};

/// Drives the connection state machine for all spaces.
pub fn drive_connection_lifecycle(
    mut commands: Commands,
    networking: Res<NetworkingThread>,
    mut spaces: Query<(Entity, &Space, &mut ConnectionState, &mut ConnectionAttempt)>,
) {
    for (entity, space, mut state, attempt) in spaces.iter_mut() {
        match *state {
            ConnectionState::Disconnected => {
                if attempt.is_ready() {
                    info!("Initiating join to space {}", space.url);
                    *state = ConnectionState::Connecting { attempt: 0 };
                    send_join_command(&networking, entity, space, &mut commands);
                }
            }
            ConnectionState::Connecting { .. } => {}
            ConnectionState::Connected => {}
            ConnectionState::Reconnecting { .. } => {
                if attempt.is_ready() {
                    info!("Retrying join to space {}", space.url);
                    *state = ConnectionState::Connecting {
                        attempt: match *state {
                            ConnectionState::Reconnecting { attempt } => attempt + 1,
                            _ => 0,
                        },
                    };
                    send_join_command(&networking, entity, space, &mut commands);
                }
            }
            ConnectionState::Failed { permanent } => {
                if permanent {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

fn send_join_command(
    networking: &NetworkingThread,
    entity: Entity,
    space: &Space,
    commands: &mut Commands,
) {
    let command = NetworkCommand::JoinSpace {
        entity,
        space_url: space.url.clone(),
    };

    if let Err(e) = networking.command_tx.send(command) {
        error!("Failed to send join command: {e:?}");
        commands
            .entity(entity)
            .insert(ConnectionState::Failed { permanent: true });
    }
}
