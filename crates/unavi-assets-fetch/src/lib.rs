//! Serves [`unavi_assets::MANIFEST`] assets to Bevy out of the iroh blob
//! store, under the `iroh://` asset source.
//!
//! Nothing is written to the asset directory: the blob store holds the only
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

/// The asset source manifest assets load from. Paired with the scheme
/// [`unavi_assets::asset_path`] emits.
const SOURCE: &str = "iroh";

pub struct UnaviAssetsFetchPlugin;

impl Plugin for UnaviAssetsFetchPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = async_channel::unbounded();

        app.register_asset_source(
            SOURCE,
            AssetSourceBuilder::new(move || Box::new(IrohAssetReader(tx.clone()))),
        )
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

fn sweep_pins(stores: Query<&LocalBlobStore, Added<LocalBlobStore>>) {
    let Ok(store) = stores.single() else {
        return;
    };

    let store = store.0.clone();
    spawn_async_task(async move {
        if let Err(err) = pin::sweep(&store).await {
            error!(?err, "failed to sweep asset pins");
        }
    });
}

#[cfg(test)]
mod tests {
    use unavi_assets::{
        DEFAULT_AVATAR,
        asset_path,
    };

    use super::*;

    #[test]
    fn manifest_paths_name_this_source() {
        assert_eq!(
            asset_path(DEFAULT_AVATAR),
            format!("{SOURCE}://{DEFAULT_AVATAR}"),
            "the path assets load by resolves to the source this plugin registers"
        );
    }
}
