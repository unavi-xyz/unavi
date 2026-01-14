use bevy::prelude::*;
use blake3::Hash;

use crate::networking::thread::{NetworkCommand, NetworkingThread};

pub mod beacon;
mod home;
mod publish_beacons;

pub struct SpacePlugin {
    pub initial_space: Option<Hash>,
}

impl Plugin for SpacePlugin {
    fn build(&self, app: &mut App) {
        if let Some(id) = self.initial_space {
            app.add_systems(Startup, move |nt: Res<NetworkingThread>| {
                info!("Joining space: {id}");
                if let Err(e) = nt.command_tx.try_send(NetworkCommand::Join(id)) {
                    error!("Error sending space join: {e:?}");
                }
            });
        } else {
            app.add_systems(Startup, home::join_home_space);
        }

        app.add_systems(FixedUpdate, publish_beacons::publish_beacons);
    }
}

#[derive(Component)]
pub struct Space(pub Hash);
