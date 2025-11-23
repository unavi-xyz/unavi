use bevy::{
    app::AppExit,
    log::{error, info},
    prelude::*,
};

use crate::space::{
    Space,
    networking::{NetworkingThread, thread::NetworkCommand},
};

pub fn handle_space_despawn(event: On<Remove, Space>, networking: Res<NetworkingThread>) {
    let entity = event.entity;

    let command = NetworkCommand::LeaveSpace { entity };

    if let Err(e) = networking.command_tx.send(command) {
        error!("Failed to send leave space command: {e:?}");
    }
}

pub fn cleanup_connections_on_exit(
    networking: Res<NetworkingThread>,
    mut exit_events: MessageReader<AppExit>,
) {
    for _ in exit_events.read() {
        info!("Sending shutdown command to networking thread");
        let _ = networking.command_tx.send(NetworkCommand::Shutdown);
    }
}
