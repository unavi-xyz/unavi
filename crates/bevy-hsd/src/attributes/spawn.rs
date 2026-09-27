use bevy::prelude::*;
use hsd::schema::spawn::SpawnAttr;
use unavi_physics::finite;

use crate::attributes::{
    ParseError,
    apply_simple,
};

#[derive(Component, Debug, Clone, Copy)]
pub struct SpawnPoint {
    pub radius: f32,
}

pub(crate) fn apply(
    commands: &mut Commands,
    prim: Entity,
    payload: Option<&[u8]>,
) -> Result<(), ParseError> {
    apply_simple::<SpawnAttr, SpawnPoint>(commands, prim, payload, |attr| {
        let radius = attr.radius as f32;
        // A non-finite or negative radius spawns at a point rather than
        // refusing the spawn point outright.
        let radius = if finite::nonneg(radius) {
            radius
        } else {
            warn!("spawn: radius must be finite and >= 0 (got {radius}); using 0");
            0.0
        };
        Some(SpawnPoint { radius })
    })
}
