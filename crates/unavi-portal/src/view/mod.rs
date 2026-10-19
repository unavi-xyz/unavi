//! Live views through open portals: a budget of render-to-texture cameras
//! mirroring the main view across the subset of portals it is close enough
//! to care about.

use bevy::prelude::*;

pub mod camera;
pub mod hue;
pub mod live;
pub mod material;
pub mod mesh;
pub mod ring;
pub mod select;
pub mod still;
pub mod visual;

/// How many portals render a live view at once, and how close the viewer
/// must be for one to earn a slot.
#[derive(Resource)]
pub struct ViewBudget {
    pub max_active:   usize,
    pub max_distance: f32,
}

impl Default for ViewBudget {
    fn default() -> Self {
        Self {
            max_active:   8,
            max_distance: 64.0,
        }
    }
}

/// Marker for a portal chosen to render a live view through an RTT camera.
#[derive(Component)]
pub struct Viewed;
