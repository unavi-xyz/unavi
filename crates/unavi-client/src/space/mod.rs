use bevy::prelude::*;
use blake3::Hash;
use loro::LoroDoc;

mod dynamic_docs;
mod home;
pub mod lifecycle;
mod publish_beacons;
mod spawn;

pub use lifecycle::JoinedSpace;

pub struct SpacePlugin;

impl Plugin for SpacePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                home::join_home_space,
                publish_beacons::publish_beacons,
                spawn::spawn_space_hsd,
                dynamic_docs::fetch_dynamic_docs,
            ),
        )
        .add_systems(
            PostUpdate,
            (lifecycle::on_space_joined, lifecycle::on_space_left),
        );
    }
}

#[derive(Component)]
pub struct Space(pub Hash);

#[derive(Component)]
pub struct SpaceDoc(pub LoroDoc);
