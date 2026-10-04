//! A prim's `reference` attribute, and the document it opens as a child.

use bevy::prelude::*;
use bevy_async::{
    AsyncWorld,
    task,
};
use bevy_iroh::store::DataStore;
use hsd::{
    attributes::reference::ReferenceAttr,
    id::DocId,
    state::{
        HsdState,
        layer::Layer,
    },
};
use iroh_docs::NamespaceId;
use unavi_store::Store;

use crate::{
    attributes::apply_simple,
    document::{
        Hsd,
        HsdDocId,
        HsdNamespace,
        SyncPeers,
    },
    feed::{
        DocFeed,
        FeedReady,
    },
    prim::{
        Prim,
        PrimOf,
    },
};

/// The decoded `reference` attribute.
#[derive(Component, Debug, Clone, Copy)]
pub struct Reference(pub DocId);

pub(crate) fn apply(
    commands: &mut Commands,
    prim: Entity,
    payload: Option<&[u8]>,
) -> Result<(), postcard::Error> {
    apply_simple::<ReferenceAttr, Reference>(commands, prim, payload, |attr| {
        Some(Reference(attr.0))
    })
}

#[derive(Component, Debug, Clone, Copy)]
pub enum ReferenceStatus {
    /// The instance spawns once the open completes.
    Opened(DocId),
    /// Past [`MAX_REF_DEPTH`], or the open failed. Not retried until
    /// [`Reference`] changes.
    Refused(DocId),
}

impl ReferenceStatus {
    #[must_use]
    pub const fn target(&self) -> DocId {
        match self {
            Self::Opened(target) | Self::Refused(target) => *target,
        }
    }
}

/// Deepest chain of references a document may open through. Bounds a
/// self-referencing document to `fan-out ^ depth` states.
pub(crate) const MAX_REF_DEPTH: usize = 8;

/// A document opened by a prim's [`Reference`]. `depth` is 1 under a root
/// document.
#[derive(Component)]
#[relationship(relationship_target = ReferenceInstances)]
pub struct ReferenceInstance {
    #[relationship]
    pub prim:   Entity,
    pub target: DocId,
    pub depth:  usize,
}

#[derive(Component, Default)]
#[relationship_target(relationship = ReferenceInstance, linked_spawn)]
pub struct ReferenceInstances(Vec<Entity>);

/// Opens each referencing prim's target as a child document.
///
/// The child's [`HsdDocId`] is [`DocId::site`], not the target, so two prims
/// referencing one document get separate ids.
pub(crate) fn open_references(
    detached: Query<Entity, (With<ReferenceStatus>, Without<Reference>)>,
    refs: Query<
        (Entity, &Prim, &Reference, &PrimOf, Option<&ReferenceStatus>),
        Or<(Changed<Reference>, Without<ReferenceStatus>)>,
    >,
    hosts: Query<(&HsdDocId, Option<&ReferenceInstance>, Option<&SyncPeers>)>,
    store: Option<Res<DataStore>>,
    async_world: Res<AsyncWorld>,
    mut commands: Commands,
) {
    for prim_ent in &detached {
        commands
            .entity(prim_ent)
            .despawn_related::<ReferenceInstances>()
            .remove::<ReferenceStatus>();
    }

    let Some(store) = store else {
        return;
    };

    for (prim_ent, prim, reference, host, status) in &refs {
        let target = reference.0;
        if let Some(status) = status {
            if status.target() == target {
                continue;
            }
            commands
                .entity(prim_ent)
                .despawn_related::<ReferenceInstances>();
        }

        let Ok((host_id, host_instance, host_peers)) = hosts.get(host.0) else {
            continue;
        };
        let peers = host_peers.cloned().unwrap_or_default();
        let site = DocId::site(host_id.0, prim.0);
        let depth = host_instance.map_or(0, |i| i.depth) + 1;

        if depth > MAX_REF_DEPTH {
            debug!(%target, depth, "reference past the depth cap is not opened");
            commands
                .entity(prim_ent)
                .insert(ReferenceStatus::Refused(target));
            continue;
        }
        commands
            .entity(prim_ent)
            .insert(ReferenceStatus::Opened(target));

        let store = store.0.clone();
        let async_world = async_world.clone();
        task::spawn(async move {
            if let Err(err) =
                open_reference(&async_world, store, target, site, depth, peers, prim_ent).await
            {
                warn!(?err, %target, "failed to open reference");
                let mark_refused = async_world
                    .commands()
                    .push(move |world: &mut World| {
                        let Ok(mut entity) = world.get_entity_mut(prim_ent) else {
                            return;
                        };
                        let still_opening = matches!(
                            entity.get::<ReferenceStatus>(),
                            Some(ReferenceStatus::Opened(t)) if *t == target
                        );
                        if still_opening {
                            entity.insert(ReferenceStatus::Refused(target));
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

// n0_future futures stay !Send on wasm and gain `Send` elsewhere.
#[cfg_attr(target_family = "wasm", allow(clippy::future_not_send))]
async fn open_reference(
    async_world: &AsyncWorld,
    store: Store,
    target: DocId,
    site: DocId,
    depth: usize,
    peers: SyncPeers,
    prim_ent: Entity,
) -> anyhow::Result<()> {
    // A target no peer has served opens empty and fills in as it syncs.
    let (doc, events) = store
        .join(NamespaceId::from(&target.0), peers.0.clone())
        .await?;

    async_world
        .commands()
        .push(move |world: &mut World| {
            let still_opening = world.get_entity(prim_ent).is_ok_and(|entity| {
                matches!(
                    entity.get::<ReferenceStatus>(),
                    Some(ReferenceStatus::Opened(t)) if *t == target
                )
            });
            if !still_opening {
                return;
            }

            // Installed before the spawn so no entry is drawn without its
            // overrides.
            let mut state = HsdState::new();
            if let Some(layer) = site_overrides(world, prim_ent) {
                state.install_reference_layer(&layer);
            }

            world.spawn((
                Hsd::new(state),
                HsdDocId(site),
                DocFeed::joined(doc.clone(), events, FeedReady::Snapshot),
                HsdNamespace(doc),
                ReferenceInstance {
                    prim: prim_ent,
                    target,
                    depth,
                },
                peers,
                ChildOf(prim_ent),
            ));
        })
        .send()
        .await?;

    Ok(())
}

/// The override layer the host document holds for `prim_ent`'s reference.
fn site_overrides(world: &World, prim_ent: Entity) -> Option<Layer> {
    let site = world.get::<Prim>(prim_ent)?.0;
    let host = world.get::<PrimOf>(prim_ent)?.0;
    let state = world.get::<Hsd>(host)?.lock()?;
    state.reference_layer_for(site).cloned()
}
