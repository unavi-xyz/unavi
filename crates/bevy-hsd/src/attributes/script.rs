//! A script has no attribute payload: the wasm component *is* its
//! `p/<prim>/script/` entry, and presence of that entry is what attaches it.

use bevy::prelude::*;
use hsd::attributes::slots;

use crate::HsdSlots;

#[derive(Component, Debug, Clone)]
pub struct HsdScript(pub Vec<u8>);

pub fn track_script(
    changed: Query<(Entity, &HsdSlots), Changed<HsdSlots>>,
    mut commands: Commands,
) {
    for (entity, slots) in &changed {
        match slots.0.get(slots::SCRIPT) {
            Some(bytes) => {
                commands.entity(entity).insert(HsdScript(bytes.clone()));
            }
            None => {
                commands.entity(entity).remove::<HsdScript>();
            }
        }
    }
}
