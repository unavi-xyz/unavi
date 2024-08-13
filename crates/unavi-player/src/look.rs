use std::f32::consts::FRAC_PI_2;

use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode, window::Window};

use crate::Player;

#[derive(Resource, Event, Debug, Default, Deref, DerefMut)]
pub struct CameraLookEvent(pub Vec2);

const PITCH_BOUND: f32 = FRAC_PI_2 - 1E-3;
const SENSITIVITY: f32 = 0.001;

pub fn read_mouse_input(
    mut look_events: EventWriter<CameraLookEvent>,
    mut look_xy: Local<Vec2>,
    mut mouse_motion_events: EventReader<MouseMotion>,
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

    *look_xy += delta;
    look_xy.y = look_xy.y.clamp(-PITCH_BOUND, PITCH_BOUND);

    look_events.send(CameraLookEvent(*look_xy));
}

const CAM_LERP_FACTOR: f32 = 30.0;

pub fn apply_camera_look(
    mut cameras: Query<&mut Transform, With<Camera>>,
    mut look_events: EventReader<CameraLookEvent>,
    mut players: Query<(&mut Transform, &Children), (With<Player>, Without<Camera>)>,
    mut target_pitch: Local<Quat>,
    mut target_yaw: Local<Quat>,
    time: Res<Time>,
) {
    for look in look_events.read() {
        *target_yaw = Quat::from_rotation_y(look.x);
        *target_pitch = Quat::from_rotation_x(look.y);
    }

    let s = time.delta_seconds() * CAM_LERP_FACTOR;

    for (mut player_tr, children) in players.iter_mut() {
        player_tr.rotation = player_tr.rotation.lerp(*target_yaw, s);

        for child in children.iter() {
            if let Ok(mut camera_tr) = cameras.get_mut(*child) {
                camera_tr.rotation = camera_tr.rotation.lerp(*target_pitch, s);
            }
        }
    }
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
