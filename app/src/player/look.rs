use super::events::{PitchEvent, YawEvent};
use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode, window::Window};

#[derive(Resource)]
pub struct MouseSettings {
    pub sensitivity: f32,
    pub yaw_pitch_roll: Vec3,
}

#[derive(Component)]
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

#[derive(Component)]
pub struct LookEntity(pub Entity);

pub fn set_look_direction(settings: Res<MouseSettings>, mut query: Query<&mut LookDirection>) {
    for mut look_direction in query.iter_mut() {
        let rotation = Quat::from_euler(
            EulerRot::YXZ,
            settings.yaw_pitch_roll.x,
            settings.yaw_pitch_roll.y,
            settings.yaw_pitch_roll.z,
        );

        look_direction.forward = rotation * Vec3::NEG_Z;
        look_direction.right = rotation * Vec3::X;
        look_direction.up = rotation * Vec3::Y;
    }
}

impl Default for MouseSettings {
    fn default() -> Self {
        Self {
            sensitivity: 1.0,
            yaw_pitch_roll: Vec3::ZERO,
        }
    }
}

const SENSITIVITY_FACTOR: f32 = 1.0 / 1000.0;
const PITCH_BOUND: f32 = std::f32::consts::FRAC_PI_2 - 1E-3;

pub fn read_mouse_input(
    windows: Query<&Window>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut settings: ResMut<MouseSettings>,
    mut pitch_events: EventWriter<PitchEvent>,
    mut yaw_events: EventWriter<YawEvent>,
) {
    let window = windows.single();
    if window.cursor.grab_mode == CursorGrabMode::None {
        return;
    }

    let mut delta = Vec2::ZERO;
    for motion in mouse_motion_events.iter() {
        delta -= motion.delta;
    }

    if delta.length_squared() > 1E-6 {
        delta *= SENSITIVITY_FACTOR * settings.sensitivity;

        settings.yaw_pitch_roll += delta.extend(0.0);

        if settings.yaw_pitch_roll.y > PITCH_BOUND {
            settings.yaw_pitch_roll.y = PITCH_BOUND;
        }

        if settings.yaw_pitch_roll.y < -PITCH_BOUND {
            settings.yaw_pitch_roll.y = -PITCH_BOUND;
        }

        pitch_events.send(PitchEvent::new(settings.yaw_pitch_roll.y));
        yaw_events.send(YawEvent::new(settings.yaw_pitch_roll.x));
    }
}

pub fn grab_mouse(
    mut windows: Query<&mut Window>,
    mouse: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
) {
    let mut window = windows.single_mut();

    if mouse.just_pressed(MouseButton::Left) {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
    }

    if key.just_pressed(KeyCode::Escape) {
        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
    }
}
