use bevy::prelude::*;
use bevy_hsd::{Stage, stage::StageData};
use loro_surgeon::Hydrate;

use crate::space::SpaceDoc;

pub fn spawn_space_stage(
    mut commands: Commands,
    to_spawn: Query<(Entity, &SpaceDoc), Without<Stage>>,
) {
    for (ent, doc) in to_spawn {
        let map = doc.0.get_map("hsd");

        match StageData::hydrate(&map.get_deep_value()) {
            Ok(stage) => {
                info!(%ent, "Spawning space stage");
                commands.entity(ent).insert(Stage(stage));
            }
            Err(err) => {
                warn!(?err, "failed to hydrate space stage");
            }
        }
    }
}
