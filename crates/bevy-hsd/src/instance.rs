use bevy::prelude::*;

use crate::{HsdDoc, asset::HsdAsset};

#[derive(Component)]
pub struct InstancedHsd(pub Handle<HsdAsset>);

pub fn instance_hsd(
    hsds: Res<Assets<HsdAsset>>,
    to_spawn: Query<(Entity, &InstancedHsd), Without<HsdDoc>>,
) {
    for (_ent, handle) in to_spawn {
        let Some(_asset) = hsds.get(&handle.0) else {
            continue;
        };

        // TODO spawn HsdDoc and HsdRecordId
    }
}
