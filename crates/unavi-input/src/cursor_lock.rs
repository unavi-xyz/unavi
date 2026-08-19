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
    // Whatever holds the input needs the cursor to point with, and holding
    // the unlocked state is what keeps the look and move that are gated on
    // `Locked` from running under it.
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
