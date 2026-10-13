use bevy::prelude::*;
use bevy_mod_openxr::exts::OxrExtensions;
use bevy_mod_xr::hand_debug_gizmos::HandGizmosPlugin;
use bevy_xr_utils::{
    actions::XRUtilsActionsPlugin,
    tracking_utils::TrackingUtilitiesPlugin,
    transform_utils::TransformUtilitiesPlugin,
};

/// The `OpenXR` extension behind the Steam Frame controller interaction
/// profile that `unavi-input` binds.
///
/// A runtime that does not advertise it logs a warning and drops it, so this is
/// safe to request everywhere.
const FRAME_CONTROLLER_EXTENSION: &[u8] = b"XR_VALVE_frame_controller_interaction\0";

/// Extensions wanted at instance creation. `OxrInitPlugin::default` enables
/// hand tracking; the `Steam Frame` controller profile is newer than the
/// `openxr` crate's typed `ExtensionSet`, so it goes through the escape hatch
/// for extension names the high-level bindings do not know about.
pub fn extensions() -> OxrExtensions {
    let mut exts = OxrExtensions::default();
    exts.enable_hand_tracking();
    exts.raw_mut()
        .other
        .push(FRAME_CONTROLLER_EXTENSION.to_vec());
    exts
}

pub struct XrPlugin;

impl Plugin for XrPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            XRUtilsActionsPlugin,
            HandGizmosPlugin,
            TrackingUtilitiesPlugin,
            TransformUtilitiesPlugin,
        ))
        .insert_resource(unavi_agent::config::InputMode::Xr);
    }
}
