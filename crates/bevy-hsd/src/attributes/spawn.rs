use bevy::prelude::*;
use hsd::attributes::spawn::SpawnAttr;
use unavi_physics::finite;

use crate::attributes::apply_simple;

#[derive(Component, Debug, Clone, Copy)]
pub struct SpawnPoint {
    pub radius: f32,
}

pub(crate) fn apply(
    commands: &mut Commands,
    prim: Entity,
    payload: Option<&[u8]>,
) -> Result<(), postcard::Error> {
    apply_simple::<SpawnAttr, SpawnPoint>(commands, prim, payload, |attr| {
        let radius = attr.radius as f32;
        let radius = if finite::nonnegative_length(radius) {
            radius
        } else {
            warn!("spawn: radius must be finite and >= 0 (got {radius}); using 0");
            0.0
        };
        Some(SpawnPoint { radius })
    })
}
