use super::events::{LookDeltaEvent, LookEvent, PitchEvent, YawEvent};
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

pub fn forward_up(settings: Res<MouseSettings>, mut query: Query<&mut LookDirection>) {
    for mut look in query.iter_mut() {
        let rotation = Quat::from_euler(
            EulerRot::YXZ,
            settings.yaw_pitch_roll.x,
            settings.yaw_pitch_roll.y,
            0.0,
        );

        look.forward = rotation * -Vec3::Z;
        look.right = rotation * Vec3::X;
        look.up = rotation * Vec3::Y;
    }
}

impl Default for MouseSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.05,
            yaw_pitch_roll: Vec3::ZERO,
        }
    }
}

const PITCH_BOUND: f32 = std::f32::consts::FRAC_PI_2 - 1E-3;

pub fn input_to_look(
    windows: Query<&Window>,
    time: Res<Time>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut settings: ResMut<MouseSettings>,
    mut pitch_events: EventWriter<PitchEvent>,
    mut yaw_events: EventWriter<YawEvent>,
    mut look_events: EventWriter<LookEvent>,
    mut look_delta_events: EventWriter<LookDeltaEvent>,
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
        delta *= settings.sensitivity * time.delta_seconds();

        settings.yaw_pitch_roll += delta.extend(0.0);

        if settings.yaw_pitch_roll.y > PITCH_BOUND {
            settings.yaw_pitch_roll.y = PITCH_BOUND;
        }

        if settings.yaw_pitch_roll.y < -PITCH_BOUND {
            settings.yaw_pitch_roll.y = -PITCH_BOUND;
        }

        look_delta_events.send(LookDeltaEvent::new(&delta.extend(0.0)));
        look_events.send(LookEvent::new(&settings.yaw_pitch_roll));
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
