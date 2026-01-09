use bevy::prelude::*;

use crate::networking::thread::{NetworkCommand, NetworkingThread};

pub fn shutdown_networking_thread(
    mut exit_events: MessageReader<AppExit>,
    nt: Res<NetworkingThread>,
) {
    for _ in exit_events.read() {
        if let Err(e) = nt.command_tx.send(NetworkCommand::Shutdown) {
            warn!("Failed to shutdown networking thread: {e:?}");
        }
    }
}
