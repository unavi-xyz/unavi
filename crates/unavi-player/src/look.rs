use std::f32::consts::FRAC_PI_2;

use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode, window::Window};

#[derive(Resource)]
pub struct LookDirection {
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
}

impl Default for LookDirection {
    fn default() -> Self {
        Self {
            forward: Vec3::Z,
            right: -Vec3::X,
            up: Vec3::Y,
        }
    }
}

#[derive(Event, Debug, Default)]
pub struct PitchEvent(pub f32);

#[derive(Event, Debug, Default)]
pub struct YawEvent(pub f32);

const PITCH_BOUND: f32 = FRAC_PI_2 - 1E-3;
const SENSITIVITY: f32 = 0.001;

pub fn read_mouse_input(
    mut look_direction: ResMut<LookDirection>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut pitch_events: EventWriter<PitchEvent>,
    mut yaw_events: EventWriter<YawEvent>,
    mut yaw_pitch: Local<Vec2>,
    windows: Query<&Window>,
) {
    if mouse_motion_events.is_empty() {
        return;
    }

    if !windows
        .iter()
        .any(|window| window.cursor.grab_mode == CursorGrabMode::Locked)
    {
        return;
    }

    let mut delta = Vec2::ZERO;

    for motion in mouse_motion_events.read() {
        delta -= motion.delta;
    }

    delta *= SENSITIVITY;

    #[cfg(target_family = "wasm")]
    {
        // TODO: Move this to a one time check

        // Adjust the sensitivity when running in Firefox.
        // I think because of incorrect values within mouse move events.
        let window = web_sys::window().unwrap();
        let navigator = window.navigator().user_agent().unwrap();
        let is_firefox = navigator.to_lowercase().contains("firefox");

        if is_firefox {
            delta *= 10.0;
        }
    }

    *yaw_pitch += delta;

    if yaw_pitch.y > PITCH_BOUND {
        yaw_pitch.y = PITCH_BOUND;
    }

    if yaw_pitch.y < -PITCH_BOUND {
        yaw_pitch.y = -PITCH_BOUND;
    }

    pitch_events.send(PitchEvent(yaw_pitch.y));
    yaw_events.send(YawEvent(yaw_pitch.x));

    let rotation = Quat::from_euler(EulerRot::YXZ, yaw_pitch.x, yaw_pitch.y, 0.0);
    look_direction.forward = rotation * Vec3::NEG_Z;
    look_direction.right = rotation * Vec3::X;
    look_direction.up = rotation * Vec3::Y;
}

pub fn grab_mouse(
    mut windows: Query<&mut Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    for mut window in windows.iter_mut() {
        if mouse.just_pressed(MouseButton::Left) {
            window.cursor.visible = false;
            window.cursor.grab_mode = CursorGrabMode::Locked;
        }

        if key.just_pressed(KeyCode::Escape) {
            window.cursor.visible = true;
            window.cursor.grab_mode = CursorGrabMode::None;
        }
    }
}
