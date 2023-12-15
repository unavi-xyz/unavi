use bevy::prelude::*;

mod connection;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<JoinWorld>()
            .add_systems(Update, (join_world, connection::open_connection));
    }
}

#[derive(Event)]
pub struct JoinWorld {
    pub did: String,
}

fn join_world(_commands: Commands, mut events: EventReader<JoinWorld>) {
    for _event in events.read() {}
}
