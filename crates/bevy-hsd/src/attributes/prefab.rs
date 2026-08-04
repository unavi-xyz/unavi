//! A prefab is a compiled `.hsdz` in the prim's `b/<prim>/prefab/` entry.
//!
//! Instancing is declarative: an instance exists because the prim carries the
//! slot, and disappears when the prim or the slot does. There is no imperative
//! spawn path and no namespace minted per loader, which is what made two peers
//! disagree on a nested document's id.

use bevy::prelude::*;
use hsd::{
    attributes::slots,
    id::BlobId,
};

use crate::HsdBulk;

#[derive(Component, Debug, Clone, Copy)]
pub struct HsdPrefab(pub BlobId);

pub fn track_prefab(changed: Query<(Entity, &HsdBulk), Changed<HsdBulk>>, mut commands: Commands) {
    for (entity, bulk) in &changed {
        match bulk.0.get(slots::PREFAB) {
            Some(hash) => {
                commands.entity(entity).insert(HsdPrefab(*hash));
            }
            None => {
                commands.entity(entity).remove::<HsdPrefab>();
            }
        }
    }
}
