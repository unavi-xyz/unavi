use bevy::prelude::*;

/// Player configuration parameters.
#[derive(Component, Clone, Debug)]
pub struct PlayerConfig {
    /// Jump strength multiplier.
    pub jump_strength: f32,
    /// Player's real height in meters.
    pub real_height: f32,
    /// Walking speed in meters per second.
    pub walk_speed: f32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            jump_strength: DEFAULT_JUMP_STRENGTH,
            real_height: DEFAULT_HEIGHT,
            walk_speed: DEFAULT_WALK_SPEED,
        }
    }
}

/// | Group            | Height        |
/// | ---------------- | ------------- |
/// | Adult Male       | 1.70 – 1.78 m |
/// | Adult Female     | 1.60 – 1.67 m |
pub const DEFAULT_HEIGHT: f32 = 1.7;

pub const DEFAULT_JUMP_STRENGTH: f32 = 1.5;
pub const DEFAULT_WALK_SPEED: f32 = 4.0;

pub const PLAYER_RADIUS: f32 = 0.5;
