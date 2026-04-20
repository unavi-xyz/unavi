use std::sync::Arc;

use bevy::prelude::*;
use bevy_hsd::{HsdDoc, HsdRecordId};
use loro::LoroDoc;

use crate::space::Space;

pub fn spawn_space_hsd(mut commands: Commands, to_spawn: Query<(Entity, &Space), Without<HsdDoc>>) {
    for (ent, space) in &to_spawn {
        info!(%ent, "Spawning space HSD");
        commands.entity(ent).insert((
            // TODO fetch from wds
            // HsdDoc(Arc::new(LoroDoc::clone(&doc.0))),
            HsdRecordId(space.0),
        ));
    }
}
