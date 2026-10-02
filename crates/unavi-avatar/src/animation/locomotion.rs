//! Per-avatar locomotion speeds.
//!
//! `unavi-agent` is the one owner of the local player's walk/sprint speed;
//! it writes this component so the blend thresholds in
//! [`weights`](super::weights) track whatever the agent is configured with
//! instead of carrying their own fixed copy. Remote avatars, which no agent
//! configures, use [`Default`].

use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct LocomotionProfile {
    pub walk_speed:   f32,
    pub sprint_speed: f32,
}

impl Default for LocomotionProfile {
    fn default() -> Self {
        Self {
            walk_speed:   4.0,
            sprint_speed: 7.0,
        }
    }
}

impl LocomotionProfile {
    /// Speed below which the avatar is treated as standing still.
    #[must_use]
    pub const fn walk_start(&self) -> f32 {
        self.walk_speed / 4.0
    }

    /// Speed at which the walk-to-sprint blend begins.
    #[must_use]
    pub const fn sprint_start(&self) -> f32 {
        f32::midpoint(self.walk_speed, self.sprint_speed)
    }

    /// Speed at which the avatar is fully sprinting.
    #[must_use]
    pub const fn sprint_end(&self) -> f32 {
        self.sprint_speed
    }
}
