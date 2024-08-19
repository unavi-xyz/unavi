use std::f32::consts::FRAC_PI_2;

use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode, window::Window};

use crate::{menu::MenuState, LocalPlayer};

#[derive(Resource, Event, Debug, Default, Deref, DerefMut)]
pub struct CameraLookEvent(pub Vec2);

const PITCH_BOUND: f32 = FRAC_PI_2 - 1E-3;
const MENU_YAW_BOUND: f32 = FRAC_PI_2 - 1E-3;
const SENSITIVITY: f32 = 0.001;

pub fn read_mouse_input(
    #[cfg(target_family = "wasm")] mut is_firefox: Local<Option<bool>>,
    menu: Res<State<MenuState>>,
    mut look_events: EventWriter<CameraLookEvent>,
    mut look_xy: Local<Vec2>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut prev_menu: Local<MenuState>,
    mut yaw_bound: Local<(f32, f32)>,
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
        // Adjust the sensitivity when running in Firefox.
        // I think because of incorrect values within mouse move events.
        if let Some(is_firefox) = *is_firefox {
            if is_firefox {
                delta *= 10.0;
            }
        } else {
            let window = web_sys::window().unwrap();
            let navigator = window.navigator().user_agent().unwrap();
            *is_firefox = Some(navigator.to_lowercase().contains("firefox"));
        }
    }

    *look_xy += delta;
    look_xy.y = look_xy.y.clamp(-PITCH_BOUND, PITCH_BOUND);

    let menu = *menu.get();

    if menu == MenuState::Open {
        if *prev_menu != menu {
            *yaw_bound = ((look_xy.x - MENU_YAW_BOUND), (look_xy.x + MENU_YAW_BOUND));
        } else {
            look_xy.x = look_xy.x.clamp(yaw_bound.0, yaw_bound.1);
        }
    }

    *prev_menu = menu;

    look_events.send(CameraLookEvent(*look_xy));
}

const CAM_LERP_FACTOR: f32 = 30.0;

pub fn apply_camera_look(
    menu: Res<State<MenuState>>,
    mut cameras: Query<&mut Transform, With<Camera>>,
    mut look_events: EventReader<CameraLookEvent>,
    mut menu_yaw: Local<Option<Quat>>,
    mut players: Query<(&mut Transform, &Children), (With<LocalPlayer>, Without<Camera>)>,
    mut target_pitch: Local<Quat>,
    mut target_yaw: Local<Quat>,
    time: Res<Time>,
) {
    for look in look_events.read() {
        *target_yaw = Quat::from_rotation_y(look.x);
        *target_pitch = Quat::from_rotation_x(look.y);
    }

    let s = time.delta_seconds() * CAM_LERP_FACTOR;
    let open = *menu.get() == MenuState::Open;

    for (mut player_tr, children) in players.iter_mut() {
        if !open {
            player_tr.rotation = player_tr.rotation.lerp(*target_yaw, s);
        }

        for child in children.iter() {
            if let Ok(mut camera_tr) = cameras.get_mut(*child) {
                let target = if open {
                    if let Some(menu_yaw) = *menu_yaw {
                        (*target_yaw * menu_yaw.inverse()) * *target_pitch
                    } else {
                        *menu_yaw = Some(*target_yaw);
                        *target_pitch
                    }
                } else {
                    if menu_yaw.is_some() {
                        *menu_yaw = None;
                    }

                    *target_pitch
                };

                camera_tr.rotation = camera_tr.rotation.lerp(target, s);
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
