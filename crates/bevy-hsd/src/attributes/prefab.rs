//! A prefab is a compiled `.hsdz` in the prim's `p/<prim>/prefab/` entry.
//!
//! Instancing is declarative: an instance exists because the prim carries the
//! slot, and disappears when the prim or the slot does.

use bevy::prelude::*;
use hsd::attributes::slots;

use crate::HsdSlots;

#[derive(Component, Debug, Clone)]
pub struct HsdPrefab(pub Vec<u8>);

pub fn track_prefab(
    changed: Query<(Entity, &HsdSlots), Changed<HsdSlots>>,
    mut commands: Commands,
) {
    for (entity, slots) in &changed {
        match slots.0.get(slots::PREFAB) {
            Some(bytes) => {
                commands.entity(entity).insert(HsdPrefab(bytes.clone()));
            }
            None => {
                commands.entity(entity).remove::<HsdPrefab>();
            }
        }
    }
}
