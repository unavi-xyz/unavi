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
pub mod capture;
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
            .init_resource::<capture::Captured>()
            .init_resource::<pointer::backend::PointerFilter>()
            // Bevy's mouse pointer is what a person clicks an overlay with:
            // it follows the cursor, where ours is aimed by a head or a hand
            // and only ever parked on the render target. Nothing of ours
            // reports a hit for it, so the world it can reach is only the UI.
            // Window picking would make the window itself a hover target,
            // which it is not.
            .insert_resource(PointerInputSettings {
                is_mouse_enabled: true,
                is_touch_enabled: false,
            })
            .insert_resource(PickingSettings {
                is_window_picking_enabled: false,
                ..default()
            })
            .add_message::<pointer::PointerPressed>()
            .add_message::<pointer::PointerReleased>()
            .add_message::<pointer::GripPressed>()
            .add_message::<pointer::GripReleased>()
            .init_state::<cursor_lock::CursorGrabState>()
            .add_observer(pointer::attach_pointers)
            .add_systems(Startup, crosshair::spawn_crosshair)
            // Before `ProcessInput`, not merely before `Backend`: that is where
            // Bevy folds `PointerInput` into `PointerPress`, so a press written
            // after it lands a frame late.
            .configure_sets(
                PreUpdate,
                InputReadSet
                    .after(InputSystems)
                    .before(PickingSystems::ProcessInput),
            )
            .add_systems(
                PreUpdate,
                (
                    capture::read,
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
            .add_systems(
                Update,
                (
                    crosshair::show_crosshair,
                    crosshair::apply_crosshair_mode,
                    cursor_lock::cursor_grab,
                ),
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
