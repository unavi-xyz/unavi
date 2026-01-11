use bevy::prelude::*;

/// Player configuration parameters.
#[derive(Component, Clone, Debug)]
pub struct PlayerConfig {
    /// User's real height, headset to ground.
    pub real_height: f32,
    /// Sprint speed in meters per second.
    pub sprint_speed: f32,
    /// Walking speed in meters per second.
    pub walk_speed: f32,
    /// VRM avatar's actual height in meters (measured from bones).
    pub vrm_height: Option<f32>,
    /// VRM avatar's shoulder width in meters (for capsule radius).
    pub vrm_radius: Option<f32>,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            real_height: DEFAULT_HEIGHT,
            sprint_speed: DEFAULT_SPRINT_SPEED,
            walk_speed: DEFAULT_WALK_SPEED,
            vrm_height: None,
            vrm_radius: None,
        }
    }
}

/// | Group            | Height        |
/// | ---------------- | ------------- |
/// | Adult Male       | 1.70 – 1.78 m |
/// | Adult Female     | 1.60 – 1.67 m |
pub const DEFAULT_HEIGHT: f32 = 1.7;

pub const DEFAULT_JUMP_STRENGTH: f32 = 1.0;
pub const DEFAULT_SPRINT_SPEED: f32 = 1.0;
pub const DEFAULT_WALK_SPEED: f32 = 0.5;

pub const PLAYER_RADIUS: f32 = 0.5;
pub const FLOAT_HEIGHT_OFFSET: f32 = 0.01;

/// World scale factor resource.
///
/// This scales world objects to match player perception.
///
/// # Formula
/// `WorldScale = real_height / vrm_height`
///
/// # Behavior
/// - If VRM is **taller** than `real_height` → scale < 1.0 → world shrinks → player feels taller
/// - If VRM is **shorter** than `real_height` → scale > 1.0 → world grows → player feels shorter
/// - If VRM matches `real_height` → scale = 1.0 → no adjustment needed
///
/// # Example
/// - `real_height` = 1.7m, `vrm_height` = 2.0m → scale = 0.85
/// - World objects at 0.85x size make tall avatar feel appropriate
#[derive(Resource, Default)]
pub struct WorldScale(pub f32);

impl WorldScale {
    #[must_use]
    pub fn new(real_height: f32, vrm_height: f32) -> Self {
        Self(real_height / vrm_height)
    }
}
