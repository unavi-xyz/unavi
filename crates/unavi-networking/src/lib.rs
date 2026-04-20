use bevy::prelude::*;

use crate::thread::spawn::NetThreadLoadState;

mod thread;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, thread::spawn::spawn_net_thread)
            .add_systems(
                FixedUpdate,
                thread::spawn::spawn_net_thread.run_if(in_state(NetThreadLoadState::Loading)),
            );
    }
}
