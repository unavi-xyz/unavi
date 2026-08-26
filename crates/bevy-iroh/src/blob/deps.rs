use bevy::prelude::*;

use crate::blob::request::BlobResponse;

#[derive(Component)]
pub struct BlobDepsLoaded;

#[derive(Component, Default)]
#[relationship_target(relationship = BlobDep, linked_spawn)]
pub struct BlobDeps(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = BlobDeps)]
pub struct BlobDep(pub Entity);

pub fn mark_blob_deps_loaded(
    mut commands: Commands,
    loading: Query<(Entity, &BlobDeps), Without<BlobDepsLoaded>>,
    responses: Query<(), (With<BlobDep>, With<BlobResponse>)>,
) {
    for (ent, deps) in loading {
        let all_loaded = deps.0.iter().all(|dep_ent| responses.contains(*dep_ent));

        if all_loaded {
            commands.entity(ent).insert(BlobDepsLoaded);
        }
    }
}
