use std::sync::Arc;

use bevy::{ecs::lifecycle::HookContext, prelude::*};
use tokio::sync::{Mutex, Notify};

use crate::{AwaitBlob, BlobRequest, BlobRequestLoading, BlobResponse};

pub fn setup(world: &mut World) {
    world.register_component_hooks::<BlobRequest>().on_remove(
        |world, HookContext { entity, .. }| {
            if let Some(loading) = world.get::<BlobRequestLoading>(entity) {
                loading.cancel.notify_one();
            }
        },
    );
}

pub fn load_blob_requests(
    mut commands: Commands,
    to_load: Query<(Entity, &BlobRequest), (Without<BlobResponse>, Without<BlobRequestLoading>)>,
) {
    for (ent, req) in to_load {
        let cancel = Arc::new(Notify::new());

        let (tx, rx) = tokio::sync::mpsc::channel(1);

        commands.trigger(AwaitBlob {
            hash: req.0,
            cancel: Arc::clone(&cancel),
            tx,
        });

        commands.entity(ent).insert(BlobRequestLoading {
            rx: Arc::new(Mutex::new(rx)),
            cancel,
        });
    }
}

pub fn recv_blob_responses(mut commands: Commands, loading: Query<(Entity, &BlobRequestLoading)>) {
    for (ent, load) in loading {
        let Ok(mut rx) = load.rx.try_lock() else {
            continue;
        };

        let Ok(bytes) = rx.try_recv() else {
            continue;
        };

        commands
            .entity(ent)
            .remove::<BlobRequestLoading>()
            .insert(BlobResponse(Some(bytes)));
    }
}
