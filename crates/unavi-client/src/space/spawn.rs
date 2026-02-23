use std::sync::Arc;

use bevy::prelude::*;
use bevy_hsd::HsdDoc;
use loro::LoroDoc;

use crate::space::SpaceDoc;

pub fn spawn_space_hsd(
    mut commands: Commands,
    to_spawn: Query<(Entity, &SpaceDoc), Without<HsdDoc>>,
) {
    for (ent, doc) in &to_spawn {
        info!(%ent, "Spawning space HSD");
        commands
            .entity(ent)
            .insert(HsdDoc(Arc::new(LoroDoc::clone(&doc.0))));
    }
}
