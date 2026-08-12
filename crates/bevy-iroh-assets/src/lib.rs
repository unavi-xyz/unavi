//! Serves content-addressed assets to Bevy from the iroh blob store, under
//! the `iroh://` asset source.
//!
//! Nothing is written to an asset directory: the blob store holds the only
//! copy, pinned against its garbage collector. Fetch, retry and error policy
//! live in bevy-wds.

use async_channel::Receiver;
use bevy::{
    asset::io::AssetSourceBuilder,
    prelude::*,
};
use bevy_wds::{
    LocalBlobStore,
    blob::request::{
        BlobRequest,
        BlobResponse,
    },
};
use bytes::Bytes;
use tokio::sync::oneshot;
use unavi_util::async_task::spawn_async_task;

use crate::reader::{
    FetchRequest,
    IrohAssetReader,
};

pub mod pin;
pub mod reader;

/// The asset source manifest assets load from.
const SOURCE: &str = "iroh";

/// A content-addressed file the store serves, named by its relative path.
#[derive(Debug, Clone, Copy)]
pub struct AssetSpec {
    pub rel_path: &'static str,
    pub hash:     &'static str,
}

/// The manifest this plugin serves, for the sweep that drops stale pins.
#[derive(Resource)]
struct Manifest(&'static [AssetSpec]);

pub struct IrohAssetsPlugin {
    manifest: &'static [AssetSpec],
}

impl IrohAssetsPlugin {
    #[must_use]
    pub const fn new(manifest: &'static [AssetSpec]) -> Self {
        Self { manifest }
    }
}

impl Plugin for IrohAssetsPlugin {
    fn build(&self, app: &mut App) {
        let manifest = self.manifest;
        let (tx, rx) = async_channel::unbounded();

        app.register_asset_source(
            SOURCE,
            AssetSourceBuilder::new(move || Box::new(IrohAssetReader::new(tx.clone(), manifest))),
        )
        .insert_resource(Manifest(manifest))
        .insert_resource(Fetches(rx))
        .add_systems(Update, (start_fetches, deliver_fetches, sweep_pins));
    }
}

/// Fetches the asset reader has handed off, awaiting a world with a store.
#[derive(Resource)]
struct Fetches(Receiver<FetchRequest>);

#[derive(Component)]
struct PendingFetch(Option<oneshot::Sender<Result<Bytes, String>>>);

fn start_fetches(mut commands: Commands, fetches: Res<Fetches>, stores: Query<&LocalBlobStore>) {
    let Ok(store) = stores.single() else {
        return;
    };

    while let Ok(fetch) = fetches.0.try_recv() {
        let store = store.0.clone();
        spawn_async_task(async move {
            if let Err(err) = pin::hold(&store, fetch.rel_path, fetch.hash).await {
                error!(path = fetch.rel_path, ?err, "failed to pin asset");
            }
        });

        commands.spawn((BlobRequest(fetch.hash), PendingFetch(Some(fetch.tx))));
    }
}

fn deliver_fetches(
    mut commands: Commands,
    mut pending: Query<(Entity, &mut PendingFetch, &BlobResponse)>,
) {
    for (entity, mut fetch, response) in &mut pending {
        let Some(tx) = fetch.0.take() else {
            continue;
        };

        let delivered = match &response.0 {
            Ok(bytes) => Ok(bytes.clone()),
            Err(err) => Err(err.to_string()),
        };

        let _ = tx.send(delivered);
        commands.entity(entity).despawn();
    }
}

fn sweep_pins(stores: Query<&LocalBlobStore, Added<LocalBlobStore>>, manifest: Res<Manifest>) {
    let Ok(store) = stores.single() else {
        return;
    };

    let store = store.0.clone();
    let manifest = manifest.0;
    spawn_async_task(async move {
        if let Err(err) = pin::sweep(&store, manifest).await {
            error!(?err, "failed to sweep asset pins");
        }
    });
}
