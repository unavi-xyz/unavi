use bevy::prelude::*;

use crate::networking::thread::{NetworkCommand, NetworkingThread};

pub fn cleanup_connections_on_exit(
    nt: Res<NetworkingThread>,
    mut exit_events: MessageReader<AppExit>,
) {
    for _ in exit_events.read() {
        if let Err(e) = nt.command_tx.send(NetworkCommand::Shutdown) {
            warn!("Failed to shutdown networking thread: {e:?}");
        }
    }
}
