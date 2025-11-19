use bevy::{log::*, prelude::*};

use super::state::{ConnectionAttempt, ConnectionState};
use crate::space::{
    Space, connect_info::ConnectInfo, networking::NetworkingThread,
    record_ref_url::parse_record_ref_url,
};

/// Drives the connection state machine for all spaces.
pub fn drive_connection_lifecycle(
    mut commands: Commands,
    networking: Res<NetworkingThread>,
    mut spaces: Query<(
        Entity,
        &Space,
        &ConnectInfo,
        &mut ConnectionState,
        &mut ConnectionAttempt,
    )>,
) {
    for (entity, space, info, mut state, attempt) in spaces.iter_mut() {
        match *state {
            ConnectionState::Disconnected => {
                if attempt.is_ready() {
                    info!("Initiating connection to {}", info.connect_url);
                    *state = ConnectionState::Connecting { attempt: 0 };
                    send_connect_command(&networking, entity, space, info, &mut commands);
                }
            }
            ConnectionState::Connecting { .. } => {}
            ConnectionState::Connected => {}
            ConnectionState::Reconnecting { .. } => {
                if attempt.is_ready() {
                    info!("Retrying connection to {}", info.connect_url);
                    *state = ConnectionState::Connecting {
                        attempt: match *state {
                            ConnectionState::Reconnecting { attempt } => attempt + 1,
                            _ => 0,
                        },
                    };
                    send_connect_command(&networking, entity, space, info, &mut commands);
                }
            }
            ConnectionState::Failed { permanent } => {
                if permanent {
                    commands.entity(entity).remove::<ConnectInfo>();
                }
            }
        }
    }
}

fn send_connect_command(
    networking: &NetworkingThread,
    entity: Entity,
    space: &Space,
    info: &ConnectInfo,
    commands: &mut Commands,
) {
    let space_id = match parse_record_ref_url(&space.url) {
        Ok(id) => id.to_string(),
        Err(e) => {
            error!("Failed to parse space URL: {e:?}");
            commands
                .entity(entity)
                .insert(ConnectionState::Failed { permanent: true });
            return;
        }
    };

    let command = super::super::NetworkCommand::Connect {
        entity,
        info: info.clone(),
        space_id,
        space_url: space.url.clone(),
    };

    if let Err(e) = networking.command_tx.send(command) {
        error!("Failed to send connect command: {e:?}");
        commands
            .entity(entity)
            .insert(ConnectionState::Failed { permanent: true });
    }
}
