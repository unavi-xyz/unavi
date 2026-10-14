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
    platform::collections::{
        HashMap,
        HashSet,
    },
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
            Mask,
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
            LAYER_PREFIX,
            SessionKey,
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
    /// What the layer was last stacked as.
    standing:            Option<Standing>,
    /// Held because a drop takes no arguments.
    pub(crate) replicas: Replicas,
}

impl SessionLayerState {
    pub(crate) const fn new(peer: EndpointId, doc: DocId, replicas: Replicas) -> Self {
        Self {
            peer,
            doc,
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

/// How the local viewer stacks `peer`'s layer on `doc`. Without replicas or a
/// view, the peer is an anonymous non-author.
fn standing(world: &World, peer: EndpointId, did: Option<&Did>, doc: DocId) -> Standing {
    let author = world
        .get_resource::<Replicas>()
        .and_then(|replicas| replicas.author(doc));
    let trust = world
        .get_resource::<SpaceView>()
        .map_or(Trust::Anonymous, |view| view.trust_of(peer));
    let permissions = world
        .get_resource::<Rules>()
        .cloned()
        .unwrap_or_default()
        .permissions(trust);
    Standing {
        peer,
        is_author: did.is_some() && author.as_ref() == did,
        trust,
        permissions,
    }
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
    let Ok(mut state) = state.lock() else {
        warn!("scene state poisoned");
        return;
    };

    state.open_tick();
    let id = layer_id(peer);
    if state.layer(&id).is_none() {
        if let Err(err) = state.add_layer(
            id.clone(),
            Merge::Latest,
            mask(standing.is_author, standing.permissions),
        ) {
            debug!(?err, %doc, "session layer not stacked");
            state.close_tick();
            return;
        }
        restack_order(&mut state, world, doc);
    }

    for (key, value, at) in opinions {
        // A guest states an opaque payload, which is what an attribute is:
        // the same encoding the document uses.
        let value = value
            .as_ref()
            .map_or_default(|bytes| Value::Attribute(bytes.clone().into()).encode());
        let entry = Entry::new(key::Key::prop(key.prim, &key.name).to_string(), value, *at);
        if let Err(err) = state.apply(&id, &entry) {
            debug!(?err, "session opinion refused by the document");
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
        let Some(replicas) = world.get_resource::<Replicas>().cloned() else {
            return;
        };
        for (peer, did) in replicas.session_layers(doc) {
            let opinions = replicas.session_opinions(peer, doc);
            compose(world, peer, did.as_ref(), doc, &opinions);
        }
    });
}

/// Orders `doc`'s session layers above everything else in its stack. A layer
/// left out of order would sit wherever it was added, possibly above the
/// author's, so on failure every session layer is masked out entirely until
/// an order succeeds. Answers whether it did.
fn restack_order(state: &mut HsdState, world: &World, doc: DocId) -> bool {
    let mut standings = world
        .get_resource::<Replicas>()
        .map_or_default(|replicas| replicas.session_layers(doc))
        .into_iter()
        .filter(|(peer, _)| state.layer(&layer_id(*peer)).is_some())
        .map(|(peer, did)| standing(world, peer, did.as_ref(), doc))
        .collect::<Vec<_>>();
    order(&mut standings);

    let session_ids = state
        .layers()
        .filter(|id| id.as_str().starts_with(LAYER_PREFIX))
        .cloned()
        .collect::<HashSet<LayerId>>();
    let mut stack = state
        .layers()
        .filter(|id| **id != LayerId::BASE && !session_ids.contains(*id))
        .cloned()
        .collect::<Vec<_>>();
    stack.extend(standings.iter().map(|standing| layer_id(standing.peer)));

    match state.reorder(&stack) {
        Ok(()) => true,
        Err(err) => {
            warn!(?err, %doc, "session layers not ordered; masking them out");
            for id in session_ids {
                if let Err(err) = state.set_mask(&id, Mask::new(|_, _| false)) {
                    warn!(?err, %doc, "session layer not masked out");
                }
            }
            false
        }
    }
}

/// Everything a layer's standing is computed from, so [`restack`] can tell in
/// constant time that none of it moved.
#[derive(Default)]
pub struct Inputs {
    me:       Option<EndpointId>,
    trust:    u64,
    bindings: u64,
    authors:  u64,
    rules:    Option<Rules>,
}

impl Inputs {
    fn of(world: &World) -> Self {
        let view = world.get_resource::<SpaceView>();
        Self {
            me:       view.map(SpaceView::me),
            trust:    view.map_or(0, |view| view.trust().generation()),
            bindings: view.map_or(0, |view| view.identity().bindings.generation()),
            authors:  world
                .get_resource::<Replicas>()
                .map_or(0, Replicas::author_generation),
            rules:    world.get_resource::<Rules>().cloned(),
        }
    }

    fn same(&self, other: &Self) -> bool {
        let rules = match (&self.rules, &other.rules) {
            (Some(a), Some(b)) => a.is_same_table(b),
            (None, None) => true,
            _ => false,
        };
        self.me == other.me
            && self.trust == other.trust
            && self.bindings == other.bindings
            && self.authors == other.authors
            && rules
    }
}

/// Restacks every layer whose standing moved.
///
/// A standing moves when an author becomes known, a peer's trust or the rules
/// change, or the local identity loads; anything else leaves every standing
/// where it was, so the pass is skipped. A standing is recorded only once its
/// document applied it, so a failure is retried on the next change.
pub fn restack(world: &mut World, mut last: Local<Option<Inputs>>) {
    let inputs = Inputs::of(world);
    if last.as_ref().is_some_and(|last| last.same(&inputs)) {
        return;
    }

    let Some(replicas) = world.get_resource::<Replicas>().cloned() else {
        return;
    };
    let mut layers = world.query::<(Entity, &SessionLayerState)>();
    let mut moved: HashMap<DocId, Vec<(Entity, Standing)>> = HashMap::new();
    let mut dids: HashMap<DocId, HashMap<EndpointId, Option<Did>>> = HashMap::new();
    for (entity, layer) in layers.iter(world) {
        let did = dids
            .entry(layer.doc)
            .or_insert_with(|| replicas.session_layers(layer.doc).into_iter().collect())
            .get(&layer.peer)
            .cloned()
            .flatten();
        let now = standing(world, layer.peer, did.as_ref(), layer.doc);
        if layer.standing != Some(now) {
            moved.entry(layer.doc).or_default().push((entity, now));
        }
    }

    let mut all_applied = true;
    for (doc, standings) in moved {
        let applied = doc_state(world, doc).is_none_or(|state| {
            let Ok(mut state) = state.lock() else {
                warn!("scene state poisoned");
                return false;
            };
            state.open_tick();
            let mut ok = true;
            for (_, now) in &standings {
                let id = layer_id(now.peer);
                if state.layer(&id).is_some()
                    && let Err(err) = state.set_mask(&id, mask(now.is_author, now.permissions))
                {
                    warn!(?err, %doc, "session layer mask not set");
                    ok = false;
                }
            }
            ok &= restack_order(&mut state, world, doc);
            state.close_tick();
            ok
        });
        if !applied {
            all_applied = false;
            continue;
        }
        for (entity, now) in standings {
            if let Some(mut layer) = world.get_mut::<SessionLayerState>(entity) {
                layer.standing = Some(now);
            }
        }
    }
    *last = all_applied.then_some(inputs);
}

/// Drops `peer`'s opinions on `keys` from `doc`'s stack, so the layers
/// beneath show through.
pub(crate) fn uncompose(world: &World, peer: EndpointId, doc: DocId, keys: &[SessionKey]) {
    let Some(state) = doc_state(world, doc) else {
        return;
    };
    let Ok(mut state) = state.lock() else {
        warn!("scene state poisoned");
        return;
    };
    let id = layer_id(peer);
    if state.layer(&id).is_none() {
        return;
    }
    state.open_tick();
    for key in keys {
        if let Err(err) = state.clear(&id, key.prim, &key.name) {
            debug!(?err, "session opinion not cleared");
        }
    }
    state.close_tick();
}
