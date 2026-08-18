use bevy::{
    input::InputSystems,
    picking::{
        PickingSettings,
        PickingSystems,
        input::PointerInputSettings,
    },
    prelude::*,
};

pub mod action;
pub mod config;
pub mod crosshair;
pub mod cursor_lock;
pub mod pointer;
pub mod source;

/// Reading every bound source into [`action::ActionState`], and turning what
/// that says into pointer presses. Runs before anything picks with them.
#[derive(SystemSet, Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct InputReadSet;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(target_family = "wasm"))]
        app.insert_resource(config::file::load());
        #[cfg(target_family = "wasm")]
        app.init_resource::<config::InputConfig>();

        app.init_resource::<action::ActionState>()
            .init_resource::<pointer::backend::PointerFilter>()
            // A bound grab is the only thing that presses a pointer, so Bevy's
            // own readers would be a second press racing ours. Window picking
            // would make the window itself a hover target, which it is not.
            .insert_resource(PointerInputSettings {
                is_mouse_enabled: false,
                is_touch_enabled: false,
            })
            .insert_resource(PickingSettings {
                is_window_picking_enabled: false,
                ..default()
            })
            .add_message::<pointer::PointerPressed>()
            .add_message::<pointer::PointerReleased>()
            .init_state::<cursor_lock::CursorGrabState>()
            .add_observer(pointer::attach_pointers)
            .add_systems(Startup, crosshair::spawn_crosshair)
            .configure_sets(
                PreUpdate,
                InputReadSet
                    .after(InputSystems)
                    .before(PickingSystems::Backend),
            )
            .add_systems(
                PreUpdate,
                (
                    action::begin_frame,
                    (
                        source::keyboard::read,
                        source::mouse::read,
                        source::gamepad::read,
                    ),
                    action::end_frame,
                    pointer::locate_pointers,
                    pointer::emit_pointer_input,
                )
                    .chain()
                    .in_set(InputReadSet),
            )
            .add_systems(
                PreUpdate,
                pointer::backend::update_hits.in_set(PickingSystems::Backend),
            )
            .add_systems(
                PreUpdate,
                pointer::relay_presses.in_set(PickingSystems::PostHover),
            )
            .add_systems(FixedUpdate, crosshair::set_crosshair_mesh)
            .add_systems(
                Update,
                (crosshair::place_crosshair, cursor_lock::cursor_grab),
            );

        #[cfg(not(target_family = "wasm"))]
        app.add_systems(
            Startup,
            source::xr::setup.before(bevy_xr_utils::actions::XRUtilsActionSystems::CreateEvents),
        )
        .add_systems(
            PreUpdate,
            source::xr::read
                .in_set(InputReadSet)
                .after(action::begin_frame)
                .before(action::end_frame)
                .after(bevy_xr_utils::actions::XRUtilsActionSystems::SyncActionStates),
        );
    }
}
