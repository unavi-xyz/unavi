use bevy::prelude::*;

mod connection;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, connection::poll_connection_thread);
    }
}
