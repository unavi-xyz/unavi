//! Serves content-addressed assets to Bevy from the iroh blob store, under
//! the `iroh://` asset source.
//!
//! Nothing is written to an asset directory. The blob store holds the only
//! copy, rooted by a document this node holds. Fetch, retry and error policy
//! live in bevy-iroh.

use std::collections::{
    HashMap,
    HashSet,
};

use async_channel::Receiver;
use bevy::{
    asset::io::AssetSourceBuilder,
    prelude::*,
};
use bevy_iroh::{
    blob::request::{
        BlobRequest,
        BlobResponse,
    },
    store::LocalStore,
};
use bytes::Bytes;
use iroh_blobs::Hash;
use tokio::sync::oneshot;
use unavi_util::async_task::spawn_async_task;
use wds::Store;

use crate::reader::{
    FetchRequest,
    IrohAssetReader,
};

pub mod reader;

/// The asset source manifest assets load from.
const SOURCE: &str = "iroh";

/// The document holding this build's manifest.
///
/// Its id is recorded device-locally rather than in the root document. A
/// manifest is tied to one build of the binary, so a pointer that followed the
/// DID across devices would hand a device running another version the wrong
/// assets.
const DOCUMENT: &str = "assets";

/// A content-addressed file the store serves, named by its relative path.
#[derive(Debug, Clone, Copy)]
pub struct AssetSpec {
    pub rel_path: &'static str,
    pub hash:     &'static str,
}

/// The manifest this plugin serves, for the reconcile that holds it.
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
        .add_systems(Update, (start_fetches, deliver_fetches, hold_manifest));
    }
}

/// Fetches the asset reader has handed off, awaiting a world with a store.
#[derive(Resource)]
struct Fetches(Receiver<FetchRequest>);

#[derive(Component)]
struct PendingFetch(Option<oneshot::Sender<Result<Bytes, String>>>);

fn start_fetches(mut commands: Commands, fetches: Res<Fetches>) {
    while let Ok(fetch) = fetches.0.try_recv() {
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

fn hold_manifest(stores: Query<&LocalStore, Added<LocalStore>>, manifest: Res<Manifest>) {
    let Ok(store) = stores.single() else {
        return;
    };

    let store = store.0.clone();
    let manifest = manifest.0;
    spawn_async_task(async move {
        if let Err(err) = reconcile(&store, manifest).await {
            error!(?err, "failed to hold manifest assets");
        }
    });
}

/// What the assets document has to change to hold exactly this build's
/// manifest.
#[derive(Debug, Default, PartialEq, Eq)]
struct Plan {
    set:    Vec<(&'static str, Hash)>,
    remove: Vec<Vec<u8>>,
}

/// A key whose hash already matches is left alone, so a restart that ships the
/// same manifest writes nothing.
///
/// A key that does not decode names no manifest path, so it is removed along
/// with the paths this build dropped.
fn plan(held: &HashMap<Vec<u8>, Hash>, manifest: &[AssetSpec]) -> anyhow::Result<Plan> {
    let mut set = Vec::new();
    for asset in manifest {
        let hash = Hash::from(blake3::Hash::from_hex(asset.hash)?);
        if held.get(asset.rel_path.as_bytes()) != Some(&hash) {
            set.push((asset.rel_path, hash));
        }
    }

    let live = manifest
        .iter()
        .map(|asset| asset.rel_path.as_bytes())
        .collect::<HashSet<_>>();
    let remove = held
        .keys()
        .filter(|key| !live.contains(key.as_slice()))
        .cloned()
        .collect();

    Ok(Plan { set, remove })
}

/// Points the assets document at exactly this build's manifest.
///
/// A held document's entries root their content against blob garbage
/// collection whether or not the content has arrived, so a manifest asset needs
/// no protection beyond an entry here.
async fn reconcile(store: &Store, manifest: &[AssetSpec]) -> anyhow::Result<()> {
    let assets = store.open_named_doc(DOCUMENT).await?;

    let held = assets
        .list(&[""])
        .await?
        .into_iter()
        .map(|entry| (entry.key().to_vec(), entry.content_hash()))
        .collect::<HashMap<_, _>>();

    let changes = plan(&held, manifest)?;

    for (rel_path, hash) in changes.set {
        // The length is unknown until the content arrives. Nothing reads it
        // back, since this document is never served and protection works off
        // the hash alone.
        assets.set_hash(rel_path, hash, 0).await?;
    }
    for key in changes.remove {
        assets.remove(key).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const AVATAR: &str = "a2f1a48db6cdf369ab510f6a6fb869d107897231b70c4920ad0357e4930c6281";
    const FONT: &str = "3a21ac778bcc91b57dc32576c6baffbcb493b78b4b6ad46b05c3d33bb5da7315";

    const MANIFEST: &[AssetSpec] = &[AssetSpec {
        rel_path: "model/default.vrm",
        hash:     AVATAR,
    }];

    fn hash(hex: &str) -> Hash {
        Hash::from(blake3::Hash::from_hex(hex).expect("hex hash"))
    }

    fn held(entries: &[(&str, &str)]) -> HashMap<Vec<u8>, Hash> {
        entries
            .iter()
            .map(|(key, hex)| ((*key).as_bytes().to_vec(), hash(hex)))
            .collect()
    }

    #[test]
    fn an_empty_document_takes_the_whole_manifest() {
        assert_eq!(
            plan(&HashMap::new(), MANIFEST).expect("plan"),
            Plan {
                set:    vec![("model/default.vrm", hash(AVATAR))],
                remove: Vec::new(),
            }
        );
    }

    #[test]
    fn a_matching_document_is_left_alone() {
        let held = held(&[("model/default.vrm", AVATAR)]);
        assert_eq!(plan(&held, MANIFEST).expect("plan"), Plan::default());
    }

    #[test]
    fn a_path_this_build_dropped_is_removed() {
        let held = held(&[("model/default.vrm", AVATAR), ("font/gone.ttf", FONT)]);

        let plan = plan(&held, MANIFEST).expect("plan");

        assert!(plan.set.is_empty(), "a matching path is not rewritten");
        assert_eq!(
            plan.remove,
            [b"font/gone.ttf".to_vec()],
            "content this build no longer ships must stop being held"
        );
    }

    #[test]
    fn a_repointed_path_is_rewritten() {
        let held = held(&[("model/default.vrm", FONT)]);

        let plan = plan(&held, MANIFEST).expect("plan");

        assert_eq!(plan.set, [("model/default.vrm", hash(AVATAR))]);
        assert!(
            plan.remove.is_empty(),
            "a path the manifest still ships is repointed, not dropped"
        );
    }

    #[test]
    fn an_undecodable_key_is_removed() {
        let held = HashMap::from([(vec![0xFF, 0xFE], hash(AVATAR))]);

        assert_eq!(
            plan(&held, MANIFEST).expect("plan").remove,
            [vec![0xFF, 0xFE]],
            "a key naming no manifest path is dropped whatever its bytes"
        );
    }
}
