use std::collections::HashMap;

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
use bevy_iroh::store::LocalStore;
use hsd::{
    id::DocId,
    package::{
        self,
        Package,
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
    HsdDocId,
    HsdNamespace,
    HsdSource,
    Prim,
    attributes::reference::HsdRef,
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
    stores: Query<&LocalStore>,
    mut commands: Commands,
) {
    let Ok(store) = stores.single() else {
        return;
    };

    for (entity, mut load) in loading {
        let Some(asset) = hsds.get(&load.handle) else {
            continue;
        };

        let store = store.0.clone();
        let package = asset.0.clone();
        let on_load = load.on_load.take();

        spawn_async_task(async move {
            if let Err(err) = build_and_instance(store, package, entity, on_load).await {
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
    store: Store,
    package: Package,
    entity: Entity,
    on_load: Option<OnLoadFn>,
) -> anyhow::Result<()> {
    // A placeholder means nothing outside the package carrying it, so every
    // document it names gets a namespace here and the references are rewritten
    // to what comes back. Minting is one pass before any write, because a
    // sub-document may reference a sibling minted after it.
    let mut minted = HashMap::new();
    let mut docs = Vec::with_capacity(package.documents.len());
    for (placeholder, entries) in package.documents {
        let doc = store.create().await?;
        minted.insert(placeholder, DocId(*doc.id().as_bytes()));
        docs.push((doc, entries));
    }

    for (doc, mut entries) in docs {
        Package::rewrite_refs(&mut entries, &minted)?;
        for (key, value) in entries {
            doc.set(key, value).await?;
        }
    }

    let doc = store.create().await?;
    let namespace = doc.id();

    let mut entries = package.entries;
    Package::rewrite_refs(&mut entries, &minted)?;
    // Package entries carry inline bytes, so there is nothing to fetch.
    for (key, value) in entries {
        doc.set(key, value).await?;
    }

    let state = document::read_state(&doc).await?;

    AsyncCommands::default()
        .push(move |world: &mut World| {
            if let Ok(mut entity) = world.get_entity_mut(entity) {
                entity.insert((
                    Hsd::new(state),
                    HsdDocId(DocId(*namespace.as_bytes())),
                    HsdNamespace(doc),
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

/// The document a prim has realized, so a change re-realizes and a removal
/// tears down.
#[derive(Component)]
pub struct RefLoaded(pub DocId);

/// Realizes each referencing prim's target as a child document.
///
/// The child's own id stays derived — [`DocId::instance`] — rather than being
/// the target's, because two prims may reference one document and everything
/// keyed by document id (the policy record, session state) is per *site*. The
/// target rides along as the namespace the child's content is read from and
/// synced through.
pub fn instance_refs(
    refs: Query<(
        Entity,
        &Prim,
        &HsdRef,
        &crate::HsdChild,
        Option<&RefLoaded>,
        Option<&Children>,
    )>,
    detached: Query<(Entity, Option<&Children>), (With<RefLoaded>, Without<HsdRef>)>,
    hsd_docs: Query<(), With<Hsd>>,
    parent_ids: Query<&HsdDocId>,
    stores: Query<&LocalStore>,
    mut commands: Commands,
) {
    for (prim_ent, children) in &detached {
        despawn_instances(&mut commands, &hsd_docs, children);
        commands.entity(prim_ent).remove::<RefLoaded>();
    }

    let Ok(store) = stores.single() else {
        return;
    };

    for (prim_ent, prim, target, doc_child, loaded, children) in &refs {
        if let Some(loaded) = loaded {
            if loaded.0 == target.0 {
                continue;
            }
            despawn_instances(&mut commands, &hsd_docs, children);
        }

        let Ok(parent_id) = parent_ids.get(doc_child.0) else {
            continue;
        };
        let site = DocId::instance(parent_id.0, prim.0);
        let target = target.0;

        commands.entity(prim_ent).insert(RefLoaded(target));

        let store = store.0.clone();
        spawn_async_task(async move {
            if let Err(err) = realize_ref(store, target, site, prim_ent).await {
                warn!(?err, %target, "failed to realize reference");
                let _ = AsyncCommands::default()
                    .push(move |world: &mut World| {
                        if let Ok(mut e) = world.get_entity_mut(prim_ent) {
                            e.remove::<RefLoaded>();
                        }
                    })
                    .send()
                    .await;
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
    prim_ent: Entity,
) -> anyhow::Result<()> {
    // A target no peer has served yet opens empty and fills in as it syncs,
    // which is the dangling case a reference has and an embedded package does
    // not.
    let doc = store.open(NamespaceId::from(&target.0)).await?;
    let state = document::read_state(&doc).await?;

    AsyncCommands::default()
        .push(move |world: &mut World| {
            let current = world
                .get_entity(prim_ent)
                .ok()
                .and_then(|e| e.get::<RefLoaded>().map(|l| l.0));
            if current == Some(target) {
                world.spawn((
                    Hsd::new(state),
                    HsdDocId(site),
                    HsdSource(target),
                    HsdNamespace(doc),
                    ChildOf(prim_ent),
                ));
            }
        })
        .send()
        .await?;

    Ok(())
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
