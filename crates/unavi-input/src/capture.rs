use bevy::prelude::*;

#[cfg(feature = "egui-filter")]
use crate::cursor_lock::CursorGrabState;

/// Whether something drawn over the world holds the input.
///
/// While it does, every action reads as released and no pointer of ours aims
/// at anything; the overlay itself is left to Bevy's own mouse pointer, which
/// no backend of ours ever answers.
#[derive(Resource, Default)]
pub struct Captured(pub bool);

/// Run condition for anything reading raw input the scene acts on, which is
/// the wheel: every other source arrives as an action, silenced at its own
/// end. An app without [`InputPlugin`](crate::InputPlugin) has nothing that
/// could take the input, so the scene has it.
#[must_use]
pub fn scene_has_input(captured: Option<Res<Captured>>) -> bool {
    captured.is_none_or(|held| !held.0)
}

/// An overlay that covers the world holds the input outright.
///
/// An `egui` window only holds it once the cursor is free: a locked cursor is
/// parked mid-screen and may sit over a window without anybody pointing at it.
pub fn read(
    #[cfg(feature = "devtools")] overlay: Option<Res<unavi_devtools::overlay::DevToolsActive>>,
    #[cfg(feature = "egui-filter")] egui: Option<Res<bevy_egui::input::EguiWantsInput>>,
    #[cfg(feature = "egui-filter")] grab: Res<State<CursorGrabState>>,
    mut captured: ResMut<Captured>,
) {
    #[cfg(feature = "devtools")]
    let by_overlay = overlay.is_some_and(|active| active.0);
    #[cfg(not(feature = "devtools"))]
    let by_overlay = false;

    #[cfg(feature = "egui-filter")]
    let by_egui = *grab.get() == CursorGrabState::Unlocked
        && egui.is_some_and(|wants| wants.wants_pointer_input());
    #[cfg(not(feature = "egui-filter"))]
    let by_egui = false;

    let held = by_overlay || by_egui;
    if captured.0 != held {
        captured.0 = held;
    }
}
