use bevy::prelude::*;
use bevy_mod_xr::{camera::XrCamera, hand_debug_gizmos::HandGizmosPlugin};
use bevy_xr_utils::{
    actions::XRUtilsActionsPlugin, tracking_utils::TrackingUtilitiesPlugin,
    transform_utils::TransformUtilitiesPlugin,
};

pub struct XrPlugin;

impl Plugin for XrPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            XRUtilsActionsPlugin,
            HandGizmosPlugin,
            TrackingUtilitiesPlugin,
            TransformUtilitiesPlugin,
        ))
        .insert_resource(unavi_agent::config::XrMode(true))
        .add_systems(FixedUpdate, set_xr_camera_layers);
    }
}

fn set_xr_camera_layers(new_xr_cameras: Query<Entity, Added<XrCamera>>) {
    for entity in new_xr_cameras {
        info!("GOT XR CAM: {entity}");
    }
}
