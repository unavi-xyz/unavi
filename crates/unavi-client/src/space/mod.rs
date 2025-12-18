use bevy::prelude::*;
use wired_data_store::RecordId;

use crate::networking::thread::{NetworkCommand, NetworkingThread};

mod startup;

pub struct SpacePlugin {
    pub initial_space: Option<RecordId>,
}

impl Plugin for SpacePlugin {
    fn build(&self, app: &mut App) {
        if let Some(id) = self.initial_space {
            app.add_systems(Startup, move |nt: Res<NetworkingThread>| {
                info!("Joining space: {id}");
                if let Err(e) = nt.command_tx.send(NetworkCommand::Join(id)) {
                    error!("Error sending space join: {e:?}");
                }
            });
        } else {
            app.add_systems(Startup, startup::join_home_space);
        }
    }
}
