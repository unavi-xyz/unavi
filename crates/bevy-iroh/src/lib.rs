use bevy::prelude::*;

pub mod endpoint;
pub mod router;

pub struct IrohPlugin;

impl Plugin for IrohPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(endpoint::on_load_endpoint)
            .add_observer(router::on_build_router)
            .add_systems(FixedUpdate, endpoint::recieve_endpoint);
    }
}
