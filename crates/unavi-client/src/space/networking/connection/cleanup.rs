use bevy::{app::AppExit, log::*, prelude::*};

use crate::space::{Space, connect_info::ConnectInfo, networking::NetworkingThread};

pub fn handle_space_disconnect(
    event: On<Remove, ConnectInfo>,
    networking: Res<NetworkingThread>,
    spaces: Query<(Entity, &Space, &ConnectInfo), With<Space>>,
) -> Result {
    let Ok((_, _, info)) = spaces.get(event.entity) else {
        Err(anyhow::anyhow!("space not found"))?
    };

    // Check if any other spaces are using this connection.
    let mut other_spaces_found = 0;
    for (other_entity, _, other_info) in spaces.iter() {
        if other_entity == event.entity {
            continue;
        }
        if other_info.connect_url != info.connect_url {
            continue;
        }
        other_spaces_found += 1;
    }

    // Only disconnect if no other spaces are using this connection.
    if other_spaces_found == 0 {
        let command = super::super::NetworkCommand::Disconnect {
            connect_url: info.connect_url.to_string(),
        };

        if let Err(e) = networking.command_tx.send(command) {
            error!("Failed to send disconnect command: {e:?}");
        }
    }

    Ok(())
}

pub fn cleanup_connections_on_exit(
    networking: Res<NetworkingThread>,
    mut exit_events: MessageReader<AppExit>,
) {
    for _ in exit_events.read() {
        info!("Sending shutdown command to networking thread");
        let _ = networking
            .command_tx
            .send(super::super::NetworkCommand::Shutdown);
    }
}
