use std::sync::mpsc::Sender;

use bevy::prelude::*;
use blake3::Hash;
use bytes::Bytes;
use iroh::EndpointId;
use wds::{Blobs, actor::Actor};

mod await_blob;

pub struct WdsPlugin;

impl Plugin for WdsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(await_blob::handle_await_blob);
    }
}

#[derive(Component)]
pub struct LocalBlobs(pub Blobs);

/// Singleton actor of the local WDS.
#[derive(Component)]
pub struct LocalActor(pub Actor);

/// Actor for a remote WDS.
/// Used for syncing, data fetching, and discovery.
#[derive(Component)]
pub struct RemoteActor(pub Actor);

#[derive(Event, Clone)]
pub struct AwaitBlob {
    pub hash: Hash,
    pub sync_from: Vec<EndpointId>,
    pub tx: Sender<Bytes>,
}
