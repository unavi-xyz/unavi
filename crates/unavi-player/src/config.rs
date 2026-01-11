use avian3d::prelude::Collider;
use bevy::prelude::*;
use bevy_tnua::TnuaConfig;

use crate::{ControlScheme, ControlSchemeConfig, PlayerEntities};

/// Player configuration parameters.
#[derive(Component, Clone, Debug)]
pub struct PlayerConfig {
    /// User's real height, headset to ground.
    pub real_height: f32,
    /// Sprint speed in meters per second.
    pub sprint_speed: f32,
    /// Walking speed in meters per second.
    pub walk_speed: f32,
    pub jump_height: f32,
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
            jump_height: DEFAULT_JUMP,
            vrm_height: None,
            vrm_radius: None,
        }
    }
}

impl PlayerConfig {
    #[must_use]
    pub fn effective_vrm_height(&self) -> f32 {
        self.vrm_height.unwrap_or(self.real_height)
    }

    #[must_use]
    pub fn effective_vrm_radius(&self) -> f32 {
        self.vrm_radius.unwrap_or(DEFAULT_RADIUS)
    }

    #[must_use]
    pub fn float_height(&self) -> f32 {
        self.effective_vrm_height() / 2.0 + self.effective_vrm_radius() + EXTRA_FLOAT_HEIGHT
    }
}

const EXTRA_FLOAT_HEIGHT: f32 = 0.01;

/// | Group            | Height        |
/// | ---------------- | ------------- |
/// | Adult Male       | 1.70 – 1.78 m |
/// | Adult Female     | 1.60 – 1.67 m |
const DEFAULT_HEIGHT: f32 = 1.7;
const DEFAULT_RADIUS: f32 = 0.5;
const DEFAULT_JUMP: f32 = 1.0;

pub const DEFAULT_SPRINT_SPEED: f32 = 0.5;
pub const DEFAULT_WALK_SPEED: f32 = 0.3;

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

// Apply player config changes to character controller.
pub fn apply_config_to_controller(
    local_player: Query<(&PlayerConfig, &PlayerEntities), Changed<PlayerConfig>>,
    mut bodies: Query<(&TnuaConfig<ControlScheme>, &mut Collider)>,
    mut controller_configs: ResMut<Assets<ControlSchemeConfig>>,
) {
    let Ok((config, entities)) = local_player.single() else {
        return;
    };

    let Ok((tnua_handle, mut collider)) = bodies.get_mut(entities.body) else {
        return;
    };
    let Some(tnua_config) = controller_configs.get_mut(tnua_handle.0.id()) else {
        return;
    };

    let shape = collider.shape_mut();
    shape.clone_from(
        Collider::capsule(config.effective_vrm_radius(), config.effective_vrm_height()).shape(),
    );

    tnua_config.jump.height = config.jump_height;

    tnua_config.basis.float_height =
        config.effective_vrm_height() / 2.0 + config.effective_vrm_radius() + EXTRA_FLOAT_HEIGHT;
}
