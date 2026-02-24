use bevy::prelude::*;
use bevy_mod_xr::hand_debug_gizmos::HandGizmosPlugin;
use bevy_xr_utils::{
    actions::XRUtilsActionsPlugin, tracking_utils::TrackingUtilitiesPlugin,
    transform_utils::TransformUtilitiesPlugin,
};

pub struct UnaviXrPlugin;

impl Plugin for UnaviXrPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            XRUtilsActionsPlugin,
            HandGizmosPlugin,
            TrackingUtilitiesPlugin,
            TransformUtilitiesPlugin,
        ));
    }
}
