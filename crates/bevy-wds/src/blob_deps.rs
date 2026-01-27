use bevy::prelude::*;

use crate::{BlobDep, BlobDeps, BlobDepsLoaded, BlobResponse};

pub fn load_blob_deps(
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
