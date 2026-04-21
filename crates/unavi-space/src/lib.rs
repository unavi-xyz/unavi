use bevy::prelude::*;
use blake3::Hash;

mod gossip;

pub struct SpacePlugin;

impl Plugin for SpacePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(gossip::spawn_gossip)
            .add_observer(gossip::on_space_add)
            .add_observer(gossip::on_space_remove);
    }
}

#[derive(Component)]
struct Space(pub Hash);
