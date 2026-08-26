use bevy::{
    prelude::*,
    window::{
        CursorGrabMode,
        CursorOptions,
        PrimaryWindow,
    },
};

use crate::capture::Captured;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum CursorGrabState {
    #[default]
    Unlocked,
    Locked,
}

pub(crate) fn cursor_grab(
    key: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    captured: Res<Captured>,
    state: Res<State<CursorGrabState>>,
    mut next_state: ResMut<NextState<CursorGrabState>>,
    mut windows: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    // Whatever holds the input needs the cursor to point with; staying
    // Unlocked also keeps look and move, gated on `Locked`, from running
    // under it.
    if captured.0 {
        if *state.get() == CursorGrabState::Locked {
            for mut cursor in &mut windows {
                cursor.visible = true;
                cursor.grab_mode = CursorGrabMode::None;
            }
            next_state.set(CursorGrabState::Unlocked);
        }
        return;
    }

    for mut cursor in &mut windows {
        if mouse.just_pressed(MouseButton::Left) {
            cursor.visible = false;
            cursor.grab_mode = CursorGrabMode::Locked;
            next_state.set(CursorGrabState::Locked);
        }

        if key.just_pressed(KeyCode::Escape) {
            cursor.visible = true;
            cursor.grab_mode = CursorGrabMode::None;
            next_state.set(CursorGrabState::Unlocked);
        }
    }
}

/// Follows the browser out of a pointer lock it dropped on its own.
///
/// Escape exits the lock without the page ever seeing the key, so a state held
/// from the last request reads `Locked` over a free cursor until a second press
/// it does hear. Only a lock the document was seen holding counts as lost,
/// since the grant lands frames after the request.
#[cfg(target_family = "wasm")]
pub(crate) fn follow_browser_lock(
    mut held: Local<bool>,
    state: Res<State<CursorGrabState>>,
    mut next_state: ResMut<NextState<CursorGrabState>>,
    mut windows: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let locked = web_sys::window()
        .and_then(|window| window.document())
        .is_some_and(|document| document.pointer_lock_element().is_some());
    let lost = *held && !locked;
    *held = locked;

    if !lost || *state.get() != CursorGrabState::Locked {
        return;
    }

    for mut cursor in &mut windows {
        cursor.visible = true;
        cursor.grab_mode = CursorGrabMode::None;
    }
    next_state.set(CursorGrabState::Unlocked);
}
