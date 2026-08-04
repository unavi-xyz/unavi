//! A script has no attribute payload: the wasm component *is* its
//! `b/<prim>/script/` entry, and presence of that entry is what attaches it.

use bevy::prelude::*;
use hsd::attributes::slots;

use crate::HsdBulk;

#[derive(Component, Debug, Clone)]
pub struct HsdScript(pub blake3::Hash);

pub fn track_script(changed: Query<(Entity, &HsdBulk), Changed<HsdBulk>>, mut commands: Commands) {
    for (entity, bulk) in &changed {
        match bulk.0.get(slots::SCRIPT) {
            Some(hash) => {
                commands
                    .entity(entity)
                    .insert(HsdScript(blake3::Hash::from_bytes(hash.0)));
            }
            None => {
                commands.entity(entity).remove::<HsdScript>();
            }
        }
    }
}
