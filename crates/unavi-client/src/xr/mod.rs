use bevy::prelude::*;

pub struct UnaviXrPlugin;

impl Plugin for UnaviXrPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            bevy_xr_utils::actions::XRUtilsActionsPlugin,
            bevy_mod_xr::hand_debug_gizmos::HandGizmosPlugin,
        ));
    }
}
