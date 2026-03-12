use std::sync::Arc;

use bevy::prelude::*;
use blake3::Hash;
use bytes::Bytes;
use tokio::sync::{
    Mutex, Notify,
    mpsc::{Receiver, Sender},
};
use wds::{Blobs, actor::Actor};

mod await_blob;
mod blob_deps;
mod blob_request;
pub mod util;

pub struct WdsPlugin;

impl Plugin for WdsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(await_blob::handle_await_blob)
            .add_systems(Startup, blob_request::setup)
            .add_systems(
                FixedUpdate,
                (
                    blob_deps::load_blob_deps,
                    blob_request::load_blob_requests,
                    blob_request::recv_blob_responses,
                ),
            );
    }
}

#[derive(Component)]
#[relationship(relationship_target = BlobDeps)]
pub struct BlobDep {
    pub owner: Entity,
}

#[derive(Component, Default)]
#[relationship_target(relationship = BlobDep, linked_spawn)]
pub struct BlobDeps(Vec<Entity>);

#[derive(Component)]
pub struct BlobDepsLoaded;

#[derive(Component)]
pub struct BlobRequest(pub Hash);

#[derive(Component)]
pub struct BlobRequestLoading {
    rx: Arc<Mutex<Receiver<Bytes>>>,
    cancel: Arc<Notify>,
}

#[derive(Component)]
pub struct BlobResponse(pub Option<Bytes>);

#[derive(Component)]
pub struct LocalBlobs(pub Blobs);

/// Singleton entity with the local WDS actor.
#[derive(Component)]
#[require(SyncTargets)]
pub struct LocalActor(pub Actor);

/// Remote actors to sync with.
#[derive(Component, Default)]
pub struct SyncTargets(pub Vec<Actor>);

#[derive(Event, Clone)]
pub struct AwaitBlob {
    pub hash: Hash,
    pub cancel: Arc<Notify>,
    pub tx: Sender<Bytes>,
}
