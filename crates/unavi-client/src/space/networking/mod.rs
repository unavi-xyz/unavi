use bevy::prelude::*;

pub mod connection;
pub mod streams;

pub use connection::lifecycle::{ConnectionInitiated, HostConnections};
pub use streams::{
    TransformChannels,
    publish::{HostPublishState, HostTransformStreams},
};

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HostConnections>()
            .init_resource::<ConnectionInitiated>()
            .init_resource::<HostTransformStreams>()
            .init_resource::<HostPublishState>()
            .add_observer(connection::cleanup::handle_space_disconnect)
            .add_systems(
                FixedUpdate,
                (
                    connection::lifecycle::drive_connection_lifecycle,
                    streams::control::apply_controls,
                    streams::publish::publish_transform_data,
                    streams::transform::apply_player_transforms,
                ),
            )
            .add_systems(Last, connection::cleanup::cleanup_connections_on_exit);
    }
}
