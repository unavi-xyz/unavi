use bevy::{
    asset::{
        AssetLoader,
        LoadContext,
        io::Reader,
    },
    prelude::*,
    reflect::TypePath,
    tasks::{
        BoxedFuture,
        ConditionalSendFuture,
    },
};
use bevy_wds::{
    LocalBlobs,
    LocalDocs,
};
use hsd::{
    id::DocId,
    package::{
        self,
        Package,
    },
};
use iroh_blobs::api::blobs::Blobs;
use iroh_docs::{
    NamespaceId,
    protocol::Docs,
};
use unavi_util::{
    async_commands::AsyncCommands,
    async_task::spawn_async_task,
};

use crate::{
    Hsd,
    HsdDocId,
    HsdNamespace,
    Prim,
    attributes::prefab::HsdPrefab,
    document,
};

/// A compiled document: one file with bytes inlined, arriving complete.
#[derive(Asset, TypePath)]
pub struct HsdAsset(pub Package);

#[derive(Default, TypePath)]
pub struct HsdLoader;

impl AssetLoader for HsdLoader {
    type Asset = HsdAsset;
    type Settings = ();
    type Error = anyhow::Error;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            Ok(HsdAsset(Package::decode(&bytes)?))
        })
    }

    fn extensions(&self) -> &[&str] {
        &[package::EXTENSION]
    }
}

pub struct OnLoadCtx {
    pub entity:    Entity,
    pub namespace: NamespaceId,
}

pub type OnLoadFn =
    Box<dyn FnOnce(OnLoadCtx) -> BoxedFuture<'static, anyhow::Result<()>> + Send + Sync>;

#[derive(Component)]
pub struct LoadHsd {
    pub handle:  Handle<HsdAsset>,
    pub on_load: Option<OnLoadFn>,
}

/// Loads a `.hsdz` into a namespace of its own, so the document has a stable
/// id from birth and can later be shared.
pub fn instance_hsd(
    hsds: Res<Assets<HsdAsset>>,
    loading: Query<(Entity, &mut LoadHsd)>,
    stores: Query<(&LocalDocs, &LocalBlobs)>,
    mut commands: Commands,
) {
    let Ok((local_docs, local_blobs)) = stores.single() else {
        return;
    };

    for (entity, mut load) in loading {
        let Some(asset) = hsds.get(&load.handle) else {
            continue;
        };

        let docs = local_docs.0.clone();
        let blobs = local_blobs.0.clone();
        let package = asset.0.clone();
        let on_load = load.on_load.take();

        spawn_async_task(async move {
            if let Err(err) = build_and_instance(docs, blobs, package, entity, on_load).await {
                error!(?err, "failed to instance hsd document");
            }
        });

        commands.entity(entity).remove::<LoadHsd>();
    }
}

// n0_future futures are intentionally !Send on wasm (single-threaded, no
// Send needed there); Send-bounded elsewhere.
#[cfg_attr(target_family = "wasm", expect(clippy::future_not_send))]
async fn build_and_instance(
    docs: Docs,
    blobs: Blobs,
    package: Package,
    entity: Entity,
    on_load: Option<OnLoadFn>,
) -> anyhow::Result<()> {
    let namespace = wds::entries::create(&docs).await?;
    let doc = wds::docs::ensure_open(&docs, namespace).await?;
    let author = wds::entries::author(&docs).await?;

    let writes = document::unpack(package, &blobs)?;
    wds::entries::apply(&doc, &blobs, author, writes).await?;

    let state = document::read_state(&doc, &blobs).await?;

    AsyncCommands::default()
        .push(move |world: &mut World| {
            if let Ok(mut entity) = world.get_entity_mut(entity) {
                entity.insert((
                    Hsd::new(state),
                    HsdDocId(DocId(*namespace.as_bytes())),
                    HsdNamespace(namespace),
                ));
            }
        })
        .send()
        .await?;

    if let Some(on_load) = on_load {
        on_load(OnLoadCtx { entity, namespace }).await?;
    }

    Ok(())
}

/// Records which prefab a prim has instanced, so a change re-instances and a
/// removal tears down.
#[derive(Component)]
pub struct PrefabLoaded(pub blake3::Hash);

/// Instancing is declarative: the instance exists because the prim carries the
/// slot.
///
/// The instance's id is derived rather than minted, so every peer computes the
/// same one and the prefab's `wired:kv` state and portal receptors converge.
pub fn instance_prefabs(
    prefabs: Query<(
        Entity,
        &Prim,
        &HsdPrefab,
        &crate::HsdChild,
        Option<&PrefabLoaded>,
        Option<&Children>,
    )>,
    detached: Query<(Entity, Option<&Children>), (With<PrefabLoaded>, Without<HsdPrefab>)>,
    hsd_docs: Query<(), With<Hsd>>,
    parent_ids: Query<&HsdDocId>,
    mut commands: Commands,
) {
    for (prim_ent, children) in &detached {
        despawn_instances(&mut commands, &hsd_docs, children);
        commands.entity(prim_ent).remove::<PrefabLoaded>();
    }

    for (prim_ent, prim, prefab, doc_child, loaded, children) in &prefabs {
        let expected = blake3::hash(&prefab.0);
        if let Some(loaded) = loaded {
            if loaded.0 == expected {
                continue;
            }
            despawn_instances(&mut commands, &hsd_docs, children);
        }

        let Ok(parent_id) = parent_ids.get(doc_child.0) else {
            continue;
        };
        let doc_id = DocId::instance(parent_id.0, prim.0);

        commands.entity(prim_ent).insert(PrefabLoaded(expected));

        let bytes = prefab.0.clone();

        spawn_async_task(async move {
            let state = match unpack_prefab(&bytes) {
                Ok(state) => state,
                Err(err) => {
                    warn!(?err, "failed to instance prefab");
                    let _ = AsyncCommands::default()
                        .push(move |world: &mut World| {
                            if let Ok(mut e) = world.get_entity_mut(prim_ent) {
                                e.remove::<PrefabLoaded>();
                            }
                        })
                        .send()
                        .await;
                    return;
                }
            };

            let _ = AsyncCommands::default()
                .push(move |world: &mut World| {
                    let current = world
                        .get_entity(prim_ent)
                        .ok()
                        .and_then(|e| e.get::<PrefabLoaded>().map(|l| l.0));
                    if current == Some(expected) {
                        world.spawn((Hsd::new(state), HsdDocId(doc_id), ChildOf(prim_ent)));
                    }
                })
                .send()
                .await;
        });
    }
}

#[cfg_attr(target_family = "wasm", expect(clippy::future_not_send))]
pub fn unpack_prefab(bytes: &[u8]) -> anyhow::Result<hsd::state::SceneState> {
    let package = Package::decode(bytes)?;
    document::unpack_into_state(package)
}

fn despawn_instances(
    commands: &mut Commands,
    hsd_docs: &Query<(), With<Hsd>>,
    children: Option<&Children>,
) {
    let Some(children) = children else {
        return;
    };
    for child in children.iter() {
        if hsd_docs.contains(child) {
            commands.entity(child).despawn();
        }
    }
}
