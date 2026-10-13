//! Stacking each peer's session layer into the document it speaks for.

use std::sync::{
    Arc,
    Mutex,
};

use bevy::{
    ecs::{
        lifecycle::HookContext,
        world::DeferredWorld,
    },
    platform::collections::HashSet,
    prelude::*,
};
use bevy_hsd::document::{
    DocIndex,
    Hsd,
    HsdDocId,
};
use hsd::{
    id::DocId,
    key,
    property::value::Value,
    state::{
        HsdState,
        entry::Entry,
        stack::{
            LayerId,
            Merge,
        },
    },
};
use iroh::EndpointId;
use unavi_policy::{
    rules::Rules,
    trust::Trust,
};
use xdid::core::did::Did;

use crate::{
    authority::SpaceView,
    replication::{
        Replicas,
        session::{
            Standing,
            layer_id,
            layers::Opinion,
            mask,
            order,
        },
    },
};

/// Holds one peer's session layer on one document for as long as the peer's
/// guard owner and the document's anchor both live.
#[derive(Component)]
#[component(on_remove = unstack)]
pub struct SessionLayerState {
    pub(crate) peer:     EndpointId,
    pub(crate) doc:      DocId,
    /// The DID the peer proved, which decides whether it is the author.
    pub(crate) did:      Option<Did>,
    /// What the layer was last stacked as.
    standing:            Option<Standing>,
    /// Held because a drop takes no arguments.
    pub(crate) replicas: Replicas,
}

impl SessionLayerState {
    pub(crate) const fn new(
        peer: EndpointId,
        doc: DocId,
        did: Option<Did>,
        replicas: Replicas,
    ) -> Self {
        Self {
            peer,
            doc,
            did,
            standing: None,
            replicas,
        }
    }
}

impl Drop for SessionLayerState {
    fn drop(&mut self) {
        self.replicas.remove_session_layer(self.peer, self.doc);
    }
}

fn unstack(world: DeferredWorld, ctx: HookContext) {
    let Some(layer) = world.get::<SessionLayerState>(ctx.entity) else {
        return;
    };
    let Some(state) = doc_state(&world, layer.doc) else {
        return;
    };
    let Ok(mut state) = state.lock() else {
        warn!("scene state poisoned");
        return;
    };
    // A document that never stacked the layer has nothing to remove.
    if state.layer(&layer_id(layer.peer)).is_some()
        && let Err(err) = state.remove_layer(&layer_id(layer.peer))
    {
        warn!(?err, "session layer not unstacked");
    }
}

/// The live state of a document in the world, or `None` for one this node is
/// not holding open.
pub(crate) fn doc_state(world: &World, doc: DocId) -> Option<Arc<Mutex<HsdState>>> {
    let entity = world.get_resource::<DocIndex>()?.get(doc)?;
    world.get::<Hsd>(entity).map(Hsd::handle)
}

/// How the local viewer stacks `peer`'s layer on `doc`.
fn standing(world: &World, peer: EndpointId, did: Option<&Did>, doc: DocId) -> Standing {
    let author = world.resource::<Replicas>().author(doc);
    let trust = world
        .get_resource::<SpaceView>()
        .map_or(Trust::Anonymous, |view| view.trust_of(peer));
    Standing {
        peer,
        is_author: did.is_some() && author.as_ref() == did,
        trust,
    }
}

fn rules(world: &World) -> Rules {
    world.get_resource::<Rules>().cloned().unwrap_or_default()
}

/// Composes `opinions` into `peer`'s layer on `doc`, stacking the layer first
/// if it is new. A document not in the world composes nothing; its layers are
/// stacked when it arrives.
pub(crate) fn compose(
    world: &World,
    peer: EndpointId,
    did: Option<&Did>,
    doc: DocId,
    opinions: &[Opinion],
) {
    let Some(state) = doc_state(world, doc) else {
        return;
    };
    let standing = standing(world, peer, did, doc);
    let rules = rules(world);
    let Ok(mut state) = state.lock() else {
        warn!("scene state poisoned");
        return;
    };

    let id = layer_id(peer);
    if state.layer(&id).is_none() {
        let mask = mask(standing.is_author, rules.permissions(standing.trust));
        if let Err(err) = state.add_layer(id.clone(), Merge::Latest, mask) {
            warn!(?err, %doc, "session layer not stacked");
            return;
        }
        restack_order(&mut state, world, doc);
    }

    state.open_tick();
    for (key, value, at) in opinions {
        // A guest states an opaque payload, which is what an attribute is:
        // the same encoding the document uses.
        let value = value
            .as_ref()
            .map_or_default(|bytes| Value::Attribute(bytes.clone().into()).encode());
        let entry = Entry::new(key::Key::prop(key.prim, &key.name).to_string(), value, *at);
        if let Err(err) = state.apply(&id, &entry) {
            warn!(?err, "session opinion refused by the document");
        }
    }
    state.close_tick();
}

/// Stacks every layer the replicas hold for `doc`, once its state exists.
pub fn stack_on_arrival(
    trigger: On<Insert, HsdDocId>,
    docs: Query<&HsdDocId>,
    mut commands: Commands,
) {
    let Ok(doc) = docs.get(trigger.entity).map(|doc| doc.0) else {
        return;
    };
    commands.queue(move |world: &mut World| {
        let replicas = world.resource::<Replicas>().clone();
        for peer in replicas.session_peers(doc) {
            let did = layer_did(world, peer, doc);
            let opinions = replicas.session_opinions(peer, doc);
            compose(world, peer, did.as_ref(), doc, &opinions);
        }
    });
}

/// The DID the guard of `peer`'s layer on `doc` recorded.
fn layer_did(world: &mut World, peer: EndpointId, doc: DocId) -> Option<Did> {
    let mut layers = world.query::<&SessionLayerState>();
    layers
        .iter(world)
        .find(|layer| layer.peer == peer && layer.doc == doc)
        .and_then(|layer| layer.did.clone())
}

/// Orders `doc`'s session layers above everything else in its stack.
fn restack_order(state: &mut HsdState, world: &World, doc: DocId) {
    let sessions = world
        .resource::<Replicas>()
        .session_peers(doc)
        .into_iter()
        .filter(|peer| state.layer(&layer_id(*peer)).is_some())
        .collect::<HashSet<_>>();
    let mut standings = Vec::with_capacity(sessions.len());
    let mut layers = world.try_query::<&SessionLayerState>();
    if let Some(layers) = layers.as_mut() {
        for layer in layers.iter(world) {
            if layer.doc == doc && sessions.contains(&layer.peer) {
                standings.push(standing(world, layer.peer, layer.did.as_ref(), doc));
            }
        }
    }
    order(&mut standings);

    let session_ids = standings
        .iter()
        .map(|standing| layer_id(standing.peer))
        .collect::<HashSet<LayerId>>();
    let mut stack = state
        .layers()
        .filter(|id| **id != LayerId::BASE && !session_ids.contains(*id))
        .cloned()
        .collect::<Vec<_>>();
    stack.extend(standings.iter().map(|standing| layer_id(standing.peer)));
    if let Err(err) = state.reorder(&stack) {
        warn!(?err, %doc, "session layers not ordered");
    }
}

/// Restacks every layer whose standing moved: an author became known, a
/// peer's trust changed, or the local identity loaded.
pub fn restack(world: &mut World) {
    let mut moved = HashSet::new();
    let mut layers = world.query::<(Entity, &SessionLayerState)>();
    let current = layers
        .iter(world)
        .map(|(entity, layer)| {
            (
                entity,
                layer.doc,
                layer.standing,
                standing(world, layer.peer, layer.did.as_ref(), layer.doc),
            )
        })
        .collect::<Vec<_>>();
    let rules = rules(world);

    for (entity, doc, was, now) in current {
        if was == Some(now) {
            continue;
        }
        if let Some(mut layer) = world.get_mut::<SessionLayerState>(entity) {
            layer.standing = Some(now);
        }
        let Some(state) = doc_state(world, doc) else {
            continue;
        };
        let Ok(mut state) = state.lock() else {
            warn!("scene state poisoned");
            continue;
        };
        let id = layer_id(now.peer);
        if state.layer(&id).is_none() {
            continue;
        }
        if let Err(err) = state.set_mask(&id, mask(now.is_author, rules.permissions(now.trust))) {
            warn!(?err, %doc, "session layer mask not set");
        }
        moved.insert(doc);
    }

    for doc in moved {
        let Some(state) = doc_state(world, doc) else {
            continue;
        };
        let Ok(mut state) = state.lock() else {
            warn!("scene state poisoned");
            continue;
        };
        restack_order(&mut state, world, doc);
    }
}
