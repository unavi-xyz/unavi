//! Serves content-addressed assets to Bevy from the blob store, under the
//! `iroh://` asset source.
//!
//! Nothing is written to an asset directory. The blob store holds the only
//! copy, rooted by a document this node holds. Fetch, retry and error policy
//! are [`crate::blob`]'s.

use std::collections::{
    HashMap,
    HashSet,
};

use async_channel::Receiver;
use bevy::{
    asset::io::AssetSourceBuilder,
    prelude::*,
};
use bevy_async::task;
use bytes::Bytes;
use iroh_blobs::Hash;
use tokio::sync::oneshot;
use unavi_store::Store;

use crate::{
    assets::reader::{
        FetchRequest,
        IrohAssetReader,
    },
    blob::{
        BlobError,
        BlobRequest,
        BlobResponse,
    },
    store::DataStore,
};

pub mod reader;

/// The asset source manifest assets load from.
const SOURCE: &str = "iroh";

/// The path Bevy loads the manifest asset at `rel_path` by, on every platform.
#[must_use]
pub fn asset_path(rel_path: &str) -> String {
    format!("{SOURCE}://{rel_path}")
}

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
    pub hash:     Hash,
    pub size:     u64,
}

/// Parses a 64-digit hex BLAKE3 hash at compile time, so a manifest with a
/// malformed hash does not build.
#[must_use]
pub const fn hex_hash(hex: &str) -> Hash {
    const fn nibble(digit: u8) -> u8 {
        match digit {
            b'0'..=b'9' => digit - b'0',
            b'a'..=b'f' => digit - b'a' + 10,
            b'A'..=b'F' => digit - b'A' + 10,
            _ => panic!("hash is not hex"),
        }
    }

    let hex = hex.as_bytes();
    assert!(hex.len() == 64, "hash is not 32 bytes");

    let mut bytes = [0; 32];
    let mut i = 0;
    while i < 32 {
        bytes[i] = (nibble(hex[2 * i]) << 4) | nibble(hex[2 * i + 1]);
        i += 1;
    }
    Hash::from_bytes(bytes)
}

/// The manifest this plugin serves, for the reconcile that holds it.
#[derive(Resource)]
struct Manifest(&'static [AssetSpec]);

/// Registers the `iroh://` source serving `manifest`, and holds the manifest's
/// content in the store.
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
        .add_systems(
            Update,
            (
                start_fetches,
                deliver_fetches,
                hold_manifest.run_if(resource_added::<DataStore>),
            ),
        );
    }
}

/// Fetches the asset reader has handed off, awaiting a world with a store.
#[derive(Resource)]
struct Fetches(Receiver<FetchRequest>);

#[derive(Component)]
struct PendingFetch(oneshot::Sender<Result<Bytes, BlobError>>);

/// A fetch waits for the store rather than failing on it: the reader hands off
/// as soon as an asset is requested, which for the startup font stack is
/// before the store has finished building.
fn start_fetches(mut commands: Commands, fetches: Res<Fetches>, store: Option<Res<DataStore>>) {
    if store.is_none() {
        return;
    }

    while let Ok(fetch) = fetches.0.try_recv() {
        commands.spawn((BlobRequest(fetch.hash), PendingFetch(fetch.tx)));
    }
}

fn deliver_fetches(
    mut commands: Commands,
    pending: Query<(Entity, &BlobResponse), With<PendingFetch>>,
) {
    for (entity, response) in &pending {
        let delivered = response.0.clone();
        commands
            .entity(entity)
            .queue(move |mut entity: EntityWorldMut| {
                if let Some(fetch) = entity.take::<PendingFetch>() {
                    fetch.0.send(delivered).ok();
                }
                entity.despawn();
            });
    }
}

fn hold_manifest(store: Res<DataStore>, manifest: Res<Manifest>) {
    let store = store.0.clone();
    let manifest = manifest.0;
    task::spawn(async move {
        if let Err(err) = reconcile(&store, manifest).await {
            error!(?err, "failed to hold manifest assets");
        }
    });
}

/// What the assets document has to change to hold exactly this build's
/// manifest.
#[derive(Debug, Default, PartialEq, Eq)]
struct Plan {
    set:    Vec<(&'static str, Hash, u64)>,
    remove: Vec<Vec<u8>>,
}

/// A key whose hash already matches is left alone, so a restart that ships the
/// same manifest writes nothing.
///
/// A key that does not decode names no manifest path, so it is removed along
/// with the paths this build dropped.
fn plan(held: &HashMap<Vec<u8>, Hash>, manifest: &[AssetSpec]) -> Plan {
    let set = manifest
        .iter()
        .filter(|asset| held.get(asset.rel_path.as_bytes()) != Some(&asset.hash))
        .map(|asset| (asset.rel_path, asset.hash, asset.size))
        .collect();

    let live = manifest
        .iter()
        .map(|asset| asset.rel_path.as_bytes())
        .collect::<HashSet<_>>();
    let remove = held
        .keys()
        .filter(|key| !live.contains(key.as_slice()))
        .cloned()
        .collect();

    Plan { set, remove }
}

/// Points the assets document at exactly this build's manifest.
///
/// A held document's entries root their content against blob garbage
/// collection whether or not the content has arrived, so a manifest asset needs
/// no protection beyond an entry here.
async fn reconcile(store: &Store, manifest: &[AssetSpec]) -> unavi_store::Result<()> {
    let assets = store.named(DOCUMENT).await?;

    let held = assets
        .list("")
        .await?
        .into_iter()
        .map(|entry| (entry.key().to_vec(), entry.content_hash()))
        .collect::<HashMap<_, _>>();

    let changes = plan(&held, manifest);

    for (rel_path, hash, size) in changes.set {
        // The entry's length is the manifest's recorded size. Protection reads
        // only the hash, but iroh-docs rejects a zero-length entry.
        assets.set_hash(rel_path, hash, size).await?;
    }
    for key in changes.remove {
        assets.remove_key(key).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const AVATAR: Hash =
        hex_hash("a2f1a48db6cdf369ab510f6a6fb869d107897231b70c4920ad0357e4930c6281");
    const FONT: Hash = hex_hash("3a21ac778bcc91b57dc32576c6baffbcb493b78b4b6ad46b05c3d33bb5da7315");

    const AVATAR_SIZE: u64 = 4_452_486;

    const MANIFEST: &[AssetSpec] = &[AssetSpec {
        rel_path: "model/default.vrm",
        hash:     AVATAR,
        size:     AVATAR_SIZE,
    }];

    fn held(entries: &[(&str, Hash)]) -> HashMap<Vec<u8>, Hash> {
        entries
            .iter()
            .map(|(key, hash)| ((*key).as_bytes().to_vec(), *hash))
            .collect()
    }

    #[test]
    fn a_hex_hash_matches_the_runtime_parse() {
        let hex = "a2f1a48db6cdf369ab510f6a6fb869d107897231b70c4920ad0357e4930c6281";
        assert_eq!(AVATAR, hex.parse::<Hash>().expect("hex hash"));
    }

    #[test]
    fn an_empty_document_takes_the_whole_manifest() {
        assert_eq!(
            plan(&HashMap::new(), MANIFEST),
            Plan {
                set:    vec![("model/default.vrm", AVATAR, AVATAR_SIZE)],
                remove: Vec::new(),
            }
        );
    }

    #[test]
    fn a_matching_document_is_left_alone() {
        let held = held(&[("model/default.vrm", AVATAR)]);
        assert_eq!(plan(&held, MANIFEST), Plan::default());
    }

    #[test]
    fn a_path_this_build_dropped_is_removed() {
        let held = held(&[("model/default.vrm", AVATAR), ("font/gone.ttf", FONT)]);

        let plan = plan(&held, MANIFEST);

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

        let plan = plan(&held, MANIFEST);

        assert_eq!(plan.set, [("model/default.vrm", AVATAR, AVATAR_SIZE)]);
        assert!(
            plan.remove.is_empty(),
            "a path the manifest still ships is repointed, not dropped"
        );
    }

    #[test]
    fn a_fetch_waits_for_the_store() {
        let (tx, rx) = async_channel::unbounded();
        let mut app = App::new();
        app.insert_resource(Fetches(rx));
        app.add_systems(Update, start_fetches);

        // The receiver stays alive, so the request is queued rather than
        // cancelled when the sender is dropped.
        let (deliver, _keep) = oneshot::channel();
        tx.send_blocking(FetchRequest {
            rel_path: "model/default.vrm",
            hash:     AVATAR,
            tx:       deliver,
        })
        .expect("send");

        app.update();

        let world = app.world_mut();
        let mut query = world.query::<&BlobRequest>();
        assert_eq!(
            query.iter(world).count(),
            0,
            "no request is dispatched before the store exists"
        );
    }
}
