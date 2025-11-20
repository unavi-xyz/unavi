use bevy::{app::AppExit, log::*, prelude::*};

use crate::space::{Space, networking::NetworkingThread};

pub fn handle_space_despawn(
    event: On<Remove, Space>,
    networking: Res<NetworkingThread>,
) -> Result {
    let entity = event.entity;

    let command = super::super::NetworkCommand::LeaveSpace { entity };

    if let Err(e) = networking.command_tx.send(command) {
        error!("Failed to send leave space command: {e:?}");
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
