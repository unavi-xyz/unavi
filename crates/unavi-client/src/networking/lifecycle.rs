use bevy::prelude::*;

use crate::networking::thread::{NetworkCommand, NetworkingThread};

pub fn shutdown_networking_thread(
    mut exit_events: MessageReader<AppExit>,
    nt: Res<NetworkingThread>,
) {
    for _ in exit_events.read() {
        if let Err(err) = nt.command_tx.try_send(NetworkCommand::Shutdown) {
            warn!(?err, "failed to shutdown networking thread");
        }
    }
}
