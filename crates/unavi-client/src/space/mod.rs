use bevy::prelude::*;
use blake3::Hash;
use loro::LoroDoc;

pub mod beacon;
mod default_stage;
mod home;
mod publish_beacons;

pub struct SpacePlugin;

impl Plugin for SpacePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (home::join_home_space, publish_beacons::publish_beacons),
        );
    }
}

#[derive(Component)]
pub struct Space(pub Hash);

#[derive(Component)]
pub struct SpaceDoc(pub LoroDoc);
