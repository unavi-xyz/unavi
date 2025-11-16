use bevy::prelude::*;

/// Player configuration parameters.
#[derive(Component, Clone, Debug)]
pub struct PlayerConfig {
    /// Jump strength multiplier.
    pub jump_strength: f32,
    /// User's perceived height in meters (what they should feel like in first-person).
    pub real_height: f32,
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
            jump_strength: DEFAULT_JUMP_STRENGTH,
            real_height: DEFAULT_HEIGHT,
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
pub const DEFAULT_WALK_SPEED: f32 = 4.0;

pub const PLAYER_RADIUS: f32 = 0.5;
pub const FLOAT_HEIGHT_OFFSET: f32 = 0.01;

/// Remote player configuration (VRM dimensions only).
#[derive(Component, Clone, Debug, Default)]
pub struct RemotePlayerConfig {
    /// VRM avatar's actual height in meters (measured from bones).
    pub vrm_height: Option<f32>,
    /// VRM avatar's shoulder width in meters (for capsule radius).
    pub vrm_radius: Option<f32>,
}

/// World scale factor resource.
/// Multiply world entities by this to make the player perceive correct scale.
/// Scale = real_height / vrm_height
#[derive(Resource, Default)]
pub struct WorldScale(pub f32);

impl WorldScale {
    pub fn new(real_height: f32, vrm_height: f32) -> Self {
        Self(real_height / vrm_height)
    }
}
