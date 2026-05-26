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
) {
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
