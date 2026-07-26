use bevy::{
    prelude::*,
    window::{
        CursorGrabMode,
        CursorOptions,
        PrimaryWindow,
    },
};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum CursorGrabState {
    #[default]
    Unlocked,
    Locked,
}

pub(crate) fn cursor_grab(
    key: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut next_state: ResMut<NextState<CursorGrabState>>,
    mut windows: Query<&mut CursorOptions, With<PrimaryWindow>>,
    #[cfg(feature = "egui-filter")] wants_input: Option<Res<bevy_egui::input::EguiWantsInput>>,
    #[cfg(feature = "devtools")] state: Res<State<CursorGrabState>>,
    #[cfg(feature = "devtools")] dev_tools: Option<Res<unavi_devtools::overlay::DevToolsActive>>,
) {
    // While the dev tools overlay is open, free the cursor for it and hold the
    // unlocked state so gameplay look/move (gated on `Locked`) stays suppressed.
    #[cfg(feature = "devtools")]
    if dev_tools.is_some_and(|d| d.0) {
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
            // Filter out input if egui wants the cursor.
            #[cfg(feature = "egui-filter")]
            if let Some(w) = &wants_input
                && w.wants_pointer_input()
            {
                continue;
            }

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
