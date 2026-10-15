//! Agent tuning and the rig's physical size.
//!
//! [`AgentConfig::set_rig`] is the one place a VRM's measured proportions
//! (untrusted: the VRM is user-supplied) are allowed to change the rig's
//! shape. Everything downstream reads [`AgentConfig::rig_shapes`], which can
//! never return a non-finite or degenerate collider.

use avian3d::prelude::Collider;
use bevy::prelude::*;
use bevy_tnua::TnuaConfig;
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
use unavi_avatar::animation::locomotion::LocomotionProfile;
use unavi_physics::{
    finite::positive_length,
    shape,
};

use crate::{
    AgentAvatar,
    ControlScheme,
    ControlSchemeConfig,
    LocalAgentEntities,
};

#[derive(Component, Clone, Debug)]
pub struct AgentConfig {
    /// Real height, headset to ground.
    pub real_height:  f32,
    pub sprint_multi: f32,
    pub walk_speed:   f32,
    pub jump_height:  f32,
    /// Measured from the avatar's VRM, once validated. `None` before an
    /// avatar has loaded, or after a measurement failed validation.
    rig:              Option<RigDimensions>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            real_height:  DEFAULT_HEIGHT,
            sprint_multi: DEFAULT_SPRINT_MULTI,
            walk_speed:   DEFAULT_WALK_SPEED,
            jump_height:  DEFAULT_JUMP,
            rig:          None,
        }
    }
}

pub const EXTRA_FLOAT_HEIGHT: f32 = 0.02;

const DEFAULT_HEIGHT: f32 = 1.7;
const DEFAULT_RADIUS: f32 = 0.4;
const DEFAULT_JUMP: f32 = 1.0;

pub const DEFAULT_SPRINT_MULTI: f32 = 1.75;
pub const DEFAULT_WALK_SPEED: f32 = 4.0;

/// Below this, a capsule could not plausibly hold a standing humanoid; a
/// value this small is a degenerate VRM (coincident bones), not an unusually
/// short avatar.
const MIN_RIG_HEIGHT: f32 = 0.3;
/// Above this, a VRM is reporting a height no real avatar or scene asset
/// needs; treated as hostile input rather than an unusually tall one.
const MAX_RIG_HEIGHT: f32 = 10.0;
/// See [`MAX_RIG_HEIGHT`].
const MIN_RIG_RADIUS: f32 = 0.05;
/// See [`MAX_RIG_HEIGHT`].
const MAX_RIG_RADIUS: f32 = 2.0;

/// A rig's body capsule size, accepted only once validated: finite, within a
/// human-plausible range, and tall enough to clear its own radius twice (the
/// capsule's two hemispheres).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RigDimensions {
    pub height: f32,
    pub radius: f32,
}

impl RigDimensions {
    /// A size nothing — VRM or config — can make unsafe, used whenever
    /// validation fails.
    pub const FALLBACK: Self = Self {
        height: DEFAULT_HEIGHT,
        radius: DEFAULT_RADIUS,
    };

    /// `None` if `height`/`radius` are non-finite, non-positive, outside a
    /// human-plausible range, or too close together for a capsule.
    #[must_use]
    pub fn new(height: f32, radius: f32) -> Option<Self> {
        let rejection = if !positive_length(height) {
            Some("height is not finite and positive")
        } else if !(MIN_RIG_HEIGHT..=MAX_RIG_HEIGHT).contains(&height) {
            Some("height is outside the plausible range")
        } else if !positive_length(radius) {
            Some("radius is not finite and positive")
        } else if !(MIN_RIG_RADIUS..=MAX_RIG_RADIUS).contains(&radius) {
            Some("radius is outside the plausible range")
        } else if height <= 2.0 * radius {
            Some("height does not clear twice the radius")
        } else {
            None
        };

        if let Some(reason) = rejection {
            warn!(
                height,
                radius, reason, "avatar geometry gives an unusable rig size, keeping defaults"
            );
            return None;
        }

        Some(Self { height, radius })
    }
}

impl AgentConfig {
    /// Replaces the rig's measured size with an already-validated
    /// [`RigDimensions`]; see [`RigDimensions::new`] for the validation a
    /// caller measuring from untrusted geometry (a VRM) must run first.
    pub const fn set_rig(&mut self, dims: RigDimensions) {
        self.rig = Some(dims);
    }

    /// The rig's current size: measured from the avatar once validated, or a
    /// size derived from `real_height` (itself re-validated, so a caller
    /// setting it directly cannot smuggle in a bad value), or
    /// [`RigDimensions::FALLBACK`].
    #[must_use]
    fn dimensions(&self) -> RigDimensions {
        self.rig
            .or_else(|| RigDimensions::new(self.real_height, DEFAULT_RADIUS))
            .unwrap_or(RigDimensions::FALLBACK)
    }

    #[must_use]
    pub fn rig_height(&self) -> f32 {
        self.dimensions().height
    }

    #[must_use]
    pub fn rig_radius(&self) -> f32 {
        self.dimensions().radius
    }

    #[must_use]
    pub fn float_height(&self) -> f32 {
        let dims = self.dimensions();
        dims.height / 2.0 + dims.radius + EXTRA_FLOAT_HEIGHT
    }

    /// The rig's body capsule and sensor shape. `dimensions()` only ever
    /// returns values [`unavi_physics::shape::rig_capsule`] accepts, so this
    /// cannot fail.
    #[must_use]
    pub fn rig_shapes(&self) -> (Collider, Collider) {
        let dims = self.dimensions();
        shape::rig_capsule(dims.radius, dims.height)
            .expect("AgentConfig::dimensions is always within rig_capsule's accepted range")
    }
}

/// Whether the local agent is driven by an XR headset or desktop input.
/// Desktop look/move read the mouse and keyboard directly; XR reads the
/// headset pose and controller sticks instead.
#[derive(Resource, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    Desktop,
    Xr,
}

impl InputMode {
    #[must_use]
    pub const fn is_xr(self) -> bool {
        matches!(self, Self::Xr)
    }
}

pub fn apply_config_to_controller(
    local_agent: Query<(&AgentConfig, &LocalAgentEntities, &AgentAvatar), Changed<AgentConfig>>,
    mut bodies: Query<(
        &TnuaConfig<ControlScheme>,
        &mut Collider,
        &mut TnuaAvian3dSensorShape,
    )>,
    mut avatars: Query<&mut LocomotionProfile>,
    mut controller_configs: ResMut<Assets<ControlSchemeConfig>>,
) {
    let Ok((config, entities, avatar)) = local_agent.single() else {
        return;
    };

    if let Ok((tnua_handle, mut collider, mut sensor)) = bodies.get_mut(entities.body)
        && let Some(mut tnua_config) = controller_configs.get_mut(tnua_handle.0.id())
    {
        let (body_shape, sensor_shape) = config.rig_shapes();
        collider.shape_mut().clone_from(body_shape.shape());
        sensor.0 = sensor_shape;

        tnua_config.jump.height = config.jump_height;
        tnua_config.basis.speed = config.walk_speed;
        tnua_config.basis.float_height = config.float_height();
    }

    if let Ok(mut profile) = avatars.get_mut(avatar.0) {
        profile.walk_speed = config.walk_speed;
        profile.sprint_speed = config.walk_speed * config.sprint_multi;
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AgentConfig,
        MAX_RIG_HEIGHT,
        MAX_RIG_RADIUS,
        MIN_RIG_HEIGHT,
        MIN_RIG_RADIUS,
        RigDimensions,
    };

    #[test]
    fn a_sane_size_is_accepted() {
        assert_eq!(
            RigDimensions::new(1.7, 0.4),
            Some(RigDimensions {
                height: 1.7,
                radius: 0.4,
            })
        );
    }

    #[test]
    fn nan_is_rejected() {
        assert_eq!(RigDimensions::new(f32::NAN, 0.4), None);
        assert_eq!(RigDimensions::new(1.7, f32::NAN), None);
    }

    #[test]
    fn zero_and_negative_are_rejected() {
        assert_eq!(RigDimensions::new(0.0, 0.4), None);
        assert_eq!(RigDimensions::new(-1.7, 0.4), None);
        assert_eq!(RigDimensions::new(1.7, 0.0), None);
        assert_eq!(RigDimensions::new(1.7, -0.4), None);
    }

    #[test]
    fn huge_values_are_rejected() {
        assert_eq!(RigDimensions::new(f32::INFINITY, 0.4), None);
        assert_eq!(RigDimensions::new(MAX_RIG_HEIGHT * 2.0, 0.4), None);
        assert_eq!(RigDimensions::new(1.7, MAX_RIG_RADIUS * 2.0), None);
    }

    #[test]
    fn a_radius_not_cleared_by_the_height_is_rejected() {
        assert_eq!(RigDimensions::new(0.5, 0.4), None);
    }

    #[test]
    fn the_boundary_values_are_accepted() {
        assert!(RigDimensions::new(MIN_RIG_HEIGHT, MIN_RIG_RADIUS).is_some());
        assert!(RigDimensions::new(MAX_RIG_HEIGHT, MAX_RIG_RADIUS * 2.0_f32.recip()).is_some());
    }

    #[test]
    fn set_rig_takes_an_already_validated_size() {
        let mut config = AgentConfig::default();
        let dims = RigDimensions::new(1.8, 0.35).expect("sane dimensions");
        config.set_rig(dims);
        assert_eq!((config.rig_height(), config.rig_radius()), (1.8, 0.35));
    }

    #[test]
    fn rig_shapes_never_panics_on_default_or_validated_config() {
        let mut config = AgentConfig::default();
        let _ = config.rig_shapes();

        config.set_rig(RigDimensions::new(1.8, 0.35).expect("sane dimensions"));
        let _ = config.rig_shapes();
    }

    #[test]
    fn a_garbage_real_height_still_yields_a_safe_rig() {
        let config = AgentConfig {
            real_height: f32::NAN,
            ..AgentConfig::default()
        };
        assert!(config.rig_height().is_finite() && config.rig_height() > 0.0);
        let _ = config.rig_shapes();
    }
}
