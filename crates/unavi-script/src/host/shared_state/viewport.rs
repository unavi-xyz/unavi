//! The local user's screen, for a script that lays panes against its edges.

use std::sync::Arc;

use bevy::prelude::*;
use parking_lot::RwLock;
use unavi_agent::{
    AgentCamera,
    LocalAgent,
    config::InputMode,
};

/// A screen in pixels, and the vertical field of view it is drawn with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Screen {
    pub width:  u32,
    pub height: u32,
    pub fov:    f32,
}

/// What the screen is now, or nothing in a headset or before the first frame
/// has drawn.
#[derive(Resource, Clone, Default)]
pub struct Viewport(Arc<RwLock<Option<Screen>>>);

impl Viewport {
    #[must_use]
    pub fn get(&self) -> Option<Screen> {
        *self.0.read()
    }
}

/// Records the screen while the local user is at a desk, and forgets it in a
/// headset, which has no screen to lay anything against.
pub fn track_viewport(
    mode: Res<InputMode>,
    agents: Query<&AgentCamera, With<LocalAgent>>,
    cameras: Query<(&Camera, &Projection)>,
    viewport: Res<Viewport>,
) {
    let now = if mode.is_xr() {
        None
    } else {
        agents.iter().find_map(|camera| {
            let (camera, projection) = cameras.get(camera.0).ok()?;
            let Projection::Perspective(perspective) = projection else {
                return None;
            };
            let size = camera.logical_viewport_size()?;
            Some(Screen {
                width:  size.x as u32,
                height: size.y as u32,
                fov:    perspective.fov,
            })
        })
    };
    if *viewport.0.read() != now {
        *viewport.0.write() = now;
    }
}
