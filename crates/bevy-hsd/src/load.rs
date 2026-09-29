use std::collections::HashMap;

use bevy::{
    asset::{
        AssetLoader,
        LoadContext,
        io::Reader,
    },
    prelude::*,
    reflect::TypePath,
    tasks::ConditionalSendFuture,
};
use bevy_iroh::store::LocalStore;
use hsd::{
    format::package::{
        self,
        Package,
    },
    id::DocId,
    state::{
        HsdState,
        layer::Layer,
    },
};
use iroh_docs::NamespaceId;
use unavi_util::{
    async_commands::AsyncCommands,
    async_task::spawn_async_task,
};
use wds::Store;

use crate::{
    Hsd,
    HsdChild,
    HsdDocId,
    HsdNamespace,
    HsdSource,
    HsdSyncPeers,
    Prim,
    attributes::reference::HsdRef,
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

#[derive(Component)]
pub struct LoadHsd {
    pub handle: Handle<HsdAsset>,
}

/// Loads a `.hsdz` into a namespace of its own, so the document has a stable
/// id from birth and can later be shared.
pub fn instance_hsd(
    hsds: Res<Assets<HsdAsset>>,
    loading: Query<(Entity, &LoadHsd)>,
    stores: Query<&LocalStore>,
    mut commands: Commands,
) {
    let Ok(store) = stores.single() else {
        return;
    };

    for (entity, load) in loading {
        let Some(asset) = hsds.get(&load.handle) else {
            continue;
        };

        let store = store.0.clone();
        let package = asset.0.clone();

        spawn_async_task(async move {
            if let Err(err) = build_and_instance(store, package, entity).await {
                error!(?err, "failed to instance hsd document");
            }
        });

        commands.entity(entity).remove::<LoadHsd>();
    }
}

// Removes every namespace `try_build_and_instance` minted, on the failure
// path only: a live entity holds the rest for as long as it exists.
//
// n0_future futures are intentionally !Send on wasm (single-threaded, no
// Send needed there); Send-bounded elsewhere.
#[cfg_attr(target_family = "wasm", expect(clippy::future_not_send))]
async fn build_and_instance(store: Store, package: Package, entity: Entity) -> anyhow::Result<()> {
    let mut minted = Vec::new();
    let result = try_build_and_instance(&store, package, entity, &mut minted).await;

    if result.is_err() {
        for ns in minted {
            if let Err(err) = store.remove(ns).await {
                error!(?err, %ns, "failed to remove a namespace minted by a failed instance");
            }
        }
    }

    result
}

async fn try_build_and_instance(
    store: &Store,
    package: Package,
    entity: Entity,
    minted_namespaces: &mut Vec<NamespaceId>,
) -> anyhow::Result<()> {
    // A placeholder means nothing outside the package carrying it, so every
    // document it names gets a namespace here and the references are rewritten
    // to what comes back. Minting is one pass before any write, because a
    // sub-document may reference a sibling minted after it.
    let mut minted = HashMap::new();
    let mut docs = Vec::with_capacity(package.documents.len());
    for (placeholder, entries) in package.documents {
        let doc = store.create().await?;
        minted_namespaces.push(doc.id());
        minted.insert(placeholder, DocId(*doc.id().as_bytes()));
        docs.push((doc, entries));
    }

    // Served so a client that fetches the instance can resolve its references;
    // a namespace outside the sync set answers every request with `NotFound`.
    for (doc, mut entries) in docs {
        Package::rewrite_refs(&mut entries, &minted)?;
        for (key, value) in entries {
            doc.set(key, value).await?;
        }
        doc.serve().await?;
    }

    let doc = store.create().await?;
    minted_namespaces.push(doc.id());
    let namespace = doc.id();

    let mut entries = package.entries;
    Package::rewrite_refs(&mut entries, &minted)?;
    for (key, value) in entries {
        doc.set(key, value).await?;
    }

    AsyncCommands::default()
        .push(move |world: &mut World| {
            if let Ok(mut entity) = world.get_entity_mut(entity) {
                entity.insert((
                    Hsd::new(HsdState::new()),
                    HsdDocId(DocId(*namespace.as_bytes())),
                    HsdNamespace(doc),
                ));
            }
        })
        .send()
        .await?;

    Ok(())
}

/// Deepest chain of references a document may realize through.
///
/// A document that transitively references itself would otherwise realize
/// `fan-out ^ depth` states; every peer computes the same cap, so every peer
/// stops in the same place.
pub(crate) const MAX_REF_DEPTH: usize = 8;

/// The document a prim has realized, so a change re-realizes and a removal
/// tears down.
#[derive(Component)]
pub(crate) struct RefLoaded(DocId);

/// A reference [`realize_refs`] did not realize, either past [`MAX_REF_DEPTH`]
/// or on a failed open. Cleared only when `HsdRef` changes, so the prim is not
/// retried every frame for an answer that cannot change.
#[derive(Component)]
pub(crate) struct RefRefused;

/// How many references deep a document sits. Absent on a document nothing
/// references, which is depth zero.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct RefDepth(usize);

/// Realizes each referencing prim's target as a child document.
///
/// The child's own id stays derived — [`DocId::site`] — rather than being
/// the target's, because two prims may reference one document and everything
/// keyed by document id (the policy record, session state) is per *site*. The
/// target rides along as the namespace the child's content is read from and
/// synced through.
pub(crate) fn realize_refs(
    refs: Query<
        (
            Entity,
            &Prim,
            &HsdRef,
            &HsdChild,
            Option<&RefLoaded>,
            Option<&Children>,
        ),
        Or<(Changed<HsdRef>, Without<RefLoaded>)>,
    >,
    detached: Query<(Entity, Option<&Children>), (With<RefLoaded>, Without<HsdRef>)>,
    hsd_docs: Query<(), With<Hsd>>,
    parents: Query<(&HsdDocId, Option<&RefDepth>, Option<&HsdSyncPeers>)>,
    stores: Query<&LocalStore>,
    mut commands: Commands,
) {
    for (prim_ent, children) in &detached {
        despawn_instances(&mut commands, &hsd_docs, children);
        commands
            .entity(prim_ent)
            .remove::<(RefLoaded, RefRefused)>();
    }

    let Ok(store) = stores.single() else {
        return;
    };

    for (prim_ent, prim, target, doc_child, loaded, children) in &refs {
        if let Some(loaded) = loaded {
            if loaded.0 == target.0 {
                // Rewritten to the value it already holds: whatever it
                // resolved to, loaded or refused, still stands.
                continue;
            }
            despawn_instances(&mut commands, &hsd_docs, children);
            commands.entity(prim_ent).remove::<RefRefused>();
        }

        let Ok((parent_id, parent_depth, parent_peers)) = parents.get(doc_child.0) else {
            continue;
        };
        let peers = parent_peers.cloned().unwrap_or_default();
        let site = DocId::site(parent_id.0, prim.0);
        let target = target.0;
        let depth = parent_depth.map_or(0, |d| d.0) + 1;

        commands.entity(prim_ent).insert(RefLoaded(target));
        if depth > MAX_REF_DEPTH {
            debug!(%target, depth, "reference past the depth cap is not realized");
            commands.entity(prim_ent).insert(RefRefused);
            continue;
        }

        let store = store.0.clone();
        spawn_async_task(async move {
            if let Err(err) = realize_ref(store, target, site, depth, peers, prim_ent).await {
                warn!(?err, %target, "failed to realize reference");
                let mark_refused = AsyncCommands::default()
                    .push(move |world: &mut World| {
                        if let Ok(mut e) = world.get_entity_mut(prim_ent) {
                            e.insert(RefRefused);
                        }
                    })
                    .send()
                    .await;
                if let Err(err) = mark_refused {
                    warn!(?err, "failed to mark a reference refused");
                }
            }
        });
    }
}

// n0_future futures are intentionally !Send on wasm (single-threaded, no
// Send needed there); Send-bounded elsewhere.
#[cfg_attr(target_family = "wasm", expect(clippy::future_not_send))]
async fn realize_ref(
    store: Store,
    target: DocId,
    site: DocId,
    depth: usize,
    peers: HsdSyncPeers,
    prim_ent: Entity,
) -> anyhow::Result<()> {
    // A target no peer has served yet opens empty and fills in as it syncs,
    // which is the dangling case a reference has and an embedded package does
    // not. An empty peer list serves the target without dialing anyone.
    let doc = store.open(NamespaceId::from(&target.0)).await?;
    doc.start_sync(peers.0.clone()).await?;

    AsyncCommands::default()
        .push(move |world: &mut World| {
            let current = world
                .get_entity(prim_ent)
                .ok()
                .and_then(|e| e.get::<RefLoaded>().map(|l| l.0));
            if current != Some(target) {
                return;
            }

            // Installed before the spawn, so no entry the feed brings in is
            // ever drawn without the overrides that speak for it.
            let mut state = HsdState::new();
            if let Some(layer) = site_overrides(world, prim_ent) {
                state.install_reference_layer(&layer);
            }

            world.spawn((
                Hsd::new(state),
                HsdDocId(site),
                HsdSource(target),
                HsdNamespace(doc),
                RefDepth(depth),
                peers,
                ChildOf(prim_ent),
            ));
        })
        .send()
        .await?;

    Ok(())
}

/// What the document holding `prim_ent` says about the prims of the document
/// that prim references.
fn site_overrides(world: &World, prim_ent: Entity) -> Option<Layer> {
    let site = world.get::<Prim>(prim_ent)?.0;
    let host = world.get::<HsdChild>(prim_ent)?.0;
    let state = world.get::<Hsd>(host)?.0.lock().ok()?;
    state.reference_layer_for(site).cloned()
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
