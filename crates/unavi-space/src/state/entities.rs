use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        Mutex,
    },
};

use bevy::prelude::*;
use bevy_hsd::document::{
    Hsd,
    HsdDocId,
    HsdNamespace,
};
use hsd::{
    id::DocId,
    key,
    property::{
        name::PropName,
        value::Value,
    },
    state::{
        HsdState,
        entry::Entry,
    },
};
use iroh::EndpointId;
use iroh_docs::NamespaceId;
use unavi_policy::{
    registry::Policy,
    space::Space,
};

use crate::{
    quota::Viewer,
    state::{
        cell::{
            Restored,
            SessionError,
            SessionKey,
            Standing,
        },
        clock,
        message::{
            SessionWrite,
            StateMsg,
        },
        replicas::{
            self,
            Replicas,
        },
    },
    view::SpaceView,
};

/// The requester's identity, if known yet, for sizing a newly-discovered
/// peer's quota.
fn current_view(world: &World) -> Option<SpaceView> {
    world.get_resource::<SpaceView>().cloned()
}

fn as_viewer(view: Option<&SpaceView>) -> Option<Viewer<'_>> {
    view.map(SpaceView::viewer)
}

#[derive(Component)]
#[relationship(relationship_target = DocStates)]
pub struct StateDoc(pub Entity);

#[derive(Component, Default)]
#[relationship_target(relationship = StateDoc, linked_spawn)]
pub struct DocStates(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = PeerStates)]
pub struct StatePeer(pub Entity);

#[derive(Component, Default)]
#[relationship_target(relationship = StatePeer, linked_spawn)]
pub struct PeerStates(Vec<Entity>);

/// The local peer's state owner, spawned once and living for the session. Local
/// state is cleaned only when its document despawns (leaving a space).
#[derive(Component)]
pub struct LocalPeer;

/// A connected remote peer's inbound-state owner, tied to its state stream.
/// Despawning it on disconnect cascades away all of that peer's state.
#[derive(Component)]
pub struct RemotePeer(pub EndpointId);

/// Generation of the state stream feeding a [`RemotePeer`] entity.
///
/// A newer stream (a canonical connection replacing a racing duplicate) takes
/// the entity over, so the superseded stream's teardown leaves it intact.
#[derive(Component)]
pub struct StreamGen(u64);

/// Claims the state entity for `peer`, reusing an existing one so overlapping
/// streams for the same peer never hold duplicate store guards.
pub fn claim_remote_peer(world: &mut World, peer: EndpointId, generation: u64) -> Entity {
    if let Some(e) = entity_by::<RemotePeer, _>(world, |r| r.0 == peer) {
        let newer = world.get::<StreamGen>(e).is_none_or(|g| generation > g.0);
        if newer {
            world.entity_mut(e).insert(StreamGen(generation));
        }
        return e;
    }
    world.spawn((RemotePeer(peer), StreamGen(generation))).id()
}

/// Despawns the peer's state entity only if `generation` still owns it.
pub fn release_remote_peer(world: &mut World, peer_ent: Entity, generation: u64) {
    if world
        .get::<StreamGen>(peer_ent)
        .is_some_and(|g| g.0 == generation)
    {
        world.despawn(peer_ent);
    }
}

/// A document tracked because some peer references it, anchoring its state
/// entities. Parented under the [`Space`] so leaving the space cascades it
/// away; unparented until the space is entered and adopts it.
#[derive(Component)]
pub struct SpaceDoc {
    pub doc:   DocId,
    pub space: DocId,
}

#[derive(Component)]
pub struct PinState {
    peer:     EndpointId,
    doc:      DocId,
    local:    bool,
    /// Held because releasing a pin can hand ownership to another peer, which
    /// re-attributes the document quota, and a drop takes no arguments.
    policy:   Policy,
    replicas: Replicas,
    /// Held for the same reason as `policy`/`replicas`: a released pin may
    /// size a newly-discovered peer's quota, which needs the local identity.
    view:     Option<SpaceView>,
}

impl PinState {
    fn register(
        policy: Policy,
        replicas: Replicas,
        view: Option<SpaceView>,
        peer: EndpointId,
        doc: DocId,
        space: DocId,
        at: u64,
        local: bool,
    ) -> Option<Self> {
        if !replicas.add_pin(&policy, as_viewer(view.as_ref()), peer, doc, space, at) {
            return None;
        }
        if local {
            replicas.broadcast(&StateMsg::Pin { doc, space, at });
        }
        Some(Self {
            peer,
            doc,
            local,
            policy,
            replicas,
            view,
        })
    }
}

impl Drop for PinState {
    fn drop(&mut self) {
        self.replicas.remove_pin(
            &self.policy,
            as_viewer(self.view.as_ref()),
            self.peer,
            self.doc,
        );
        if self.local {
            self.replicas.broadcast(&StateMsg::Unpin { doc: self.doc });
        }
    }
}

#[derive(Component)]
pub struct HoldState {
    peer:     EndpointId,
    doc:      DocId,
    local:    bool,
    /// Held because a drop takes no arguments.
    replicas: Replicas,
}

impl HoldState {
    fn apply(
        policy: &Policy,
        replicas: &Replicas,
        viewer: Option<Viewer>,
        peer: EndpointId,
        doc: DocId,
        space: DocId,
        at: u64,
        local: bool,
    ) -> bool {
        let ok = replicas.add_hold(policy, viewer, peer, doc, space, at);
        if ok && local {
            replicas.broadcast(&StateMsg::Hold { doc, space, at });
        }
        ok
    }

    fn register(
        policy: &Policy,
        replicas: &Replicas,
        viewer: Option<Viewer>,
        peer: EndpointId,
        doc: DocId,
        space: DocId,
        at: u64,
        local: bool,
    ) -> Option<Self> {
        Self::apply(policy, replicas, viewer, peer, doc, space, at, local).then_some(Self {
            peer,
            doc,
            local,
            replicas: replicas.clone(),
        })
    }
}

impl Drop for HoldState {
    fn drop(&mut self) {
        self.replicas.remove_hold(self.peer, self.doc);
        if self.local {
            self.replicas
                .broadcast(&StateMsg::ReleaseHold { doc: self.doc });
        }
    }
}

/// Holds one session cell for as long as the document anchoring it lives.
#[derive(Component)]
pub struct CellState {
    doc:      DocId,
    key:      SessionKey,
    /// Held because a drop takes no arguments.
    replicas: Replicas,
}

impl Drop for CellState {
    fn drop(&mut self) {
        // Nothing goes out. An opinion belongs to the document, so it outlives
        // the peer that wrote it; an explicit delete propagates as a blocked
        // key at write time instead.
        self.replicas
            .remove_session(self.doc, self.key.prim, &self.key.name);
    }
}

fn entity_by<C: Component, F: Fn(&C) -> bool>(world: &mut World, pred: F) -> Option<Entity> {
    let mut query = world.query::<(Entity, &C)>();
    query.iter(world).find(|(_, c)| pred(c)).map(|(e, _)| e)
}

fn space_entity(world: &mut World, space: DocId) -> Option<Entity> {
    entity_by::<Space, _>(world, |s| s.doc_id() == space)
}

/// Resolves the entity anchoring `doc`, spawning a [`SpaceDoc`] tracker when
/// nothing exists yet. Trackers for spaces not yet entered stay unparented so
/// their state is kept until the space is joined.
fn doc_anchor(world: &mut World, doc: DocId, space: DocId) -> Entity {
    if let Some(e) = space_entity(world, doc) {
        return e;
    }
    if let Some(e) = entity_by::<HsdNamespace, _>(world, |r| r.0.id() == NamespaceId::from(&doc.0))
    {
        return e;
    }
    if let Some(e) = entity_by::<SpaceDoc, _>(world, |d| d.doc == doc) {
        return e;
    }
    let space_ent = space_entity(world, space);
    let tracker = world.spawn(SpaceDoc { doc, space }).id();
    if let Some(space_ent) = space_ent {
        world.entity_mut(tracker).insert(ChildOf(space_ent));
    }
    tracker
}

fn local_peer_entity(world: &mut World) -> Entity {
    if let Some(e) = entity_by::<LocalPeer, _>(world, |_| true) {
        return e;
    }
    world.spawn(LocalPeer).id()
}

/// Finds an existing state entity of `C` owned by `peer_ent` matching `pred`.
fn find_state<C: Component, F: Fn(&C) -> bool>(
    world: &World,
    peer_ent: Entity,
    pred: F,
) -> Option<Entity> {
    world
        .get::<PeerStates>(peer_ent)?
        .iter()
        .find(|e| world.get::<C>(*e).is_some_and(&pred))
}

/// Finds the doc-anchored cell guard for `key`, if one exists.
fn find_cell(world: &World, anchor: Entity, doc: DocId, key: &SessionKey) -> Option<Entity> {
    world.get::<DocStates>(anchor)?.iter().find(|e| {
        world
            .get::<CellState>(*e)
            .is_some_and(|c| c.doc == doc && &c.key == key)
    })
}

fn spawn_pin(
    world: &mut World,
    peer_ent: Entity,
    peer: EndpointId,
    doc: DocId,
    space: DocId,
    at: u64,
    local: bool,
) -> bool {
    if find_state::<PinState, _>(world, peer_ent, |p| p.doc == doc).is_some() {
        return true;
    }
    let anchor = doc_anchor(world, doc, space);
    let policy = world.resource::<Policy>().clone();
    let replicas = world.resource::<Replicas>().clone();
    let view = current_view(world);
    let Some(state) = PinState::register(policy, replicas, view, peer, doc, space, at, local)
    else {
        warn!("pin on doc {doc} refused by quota");
        return false;
    };
    world.spawn((state, StateDoc(anchor), StatePeer(peer_ent)));
    true
}

fn spawn_hold(
    world: &mut World,
    peer_ent: Entity,
    peer: EndpointId,
    doc: DocId,
    space: DocId,
    at: u64,
    local: bool,
) {
    let policy = world.resource::<Policy>().clone();
    let replicas = world.resource::<Replicas>().clone();
    let view = current_view(world);
    let viewer = as_viewer(view.as_ref());
    if find_state::<HoldState, _>(world, peer_ent, |a| a.doc == doc).is_some() {
        HoldState::apply(&policy, &replicas, viewer, peer, doc, space, at, local);
        return;
    }
    let anchor = doc_anchor(world, doc, space);
    let Some(state) = HoldState::register(&policy, &replicas, viewer, peer, doc, space, at, local)
    else {
        return;
    };
    world.spawn((state, StateDoc(anchor), StatePeer(peer_ent)));
}

fn clear_hold(world: &mut World, peer_ent: Entity, doc: DocId) {
    if let Some(e) = find_state::<HoldState, _>(world, peer_ent, |a| a.doc == doc) {
        world.despawn(e);
    }
}

fn clear_pin(world: &mut World, peer_ent: Entity, doc: DocId) {
    if let Some(e) = find_state::<PinState, _>(world, peer_ent, |p| p.doc == doc) {
        world.despawn(e);
    }
}

/// Applies a batch of session opinions: records them, composes them into the
/// document they speak for, and broadcasts them if they are this peer's.
///
/// One batch, one write boundary. A script's tick flushes its whole dirty set
/// through here, and a half-applied tick can render — so the document's events
/// are withheld until every write in the batch has landed.
fn set_session(
    world: &mut World,
    peer: EndpointId,
    doc: DocId,
    space: DocId,
    writes: Vec<SessionWrite>,
    at: u64,
    local: bool,
) -> Result<(), SessionError> {
    let named: Vec<(SessionWrite, PropName)> = writes
        .into_iter()
        .map(|w| valid_name(&w.name).map(|name| (w, name)))
        .collect::<Result<_, _>>()?;

    let anchor = doc_anchor(world, doc, space);
    let policy = world.resource::<Policy>().clone();
    let replicas = world.resource::<Replicas>().clone();
    let view = current_view(world);

    for (write, name) in &named {
        replicas.add_session(
            &policy,
            as_viewer(view.as_ref()),
            peer,
            doc,
            space,
            SessionKey {
                prim: write.prim,
                name: name.clone(),
            },
            write.value.clone(),
            at,
        )?;
    }

    compose_session(world, doc, &named, at);

    if local {
        let writes = named.iter().map(|(w, _)| w.clone()).collect();
        replicas.broadcast(&StateMsg::Session {
            doc,
            space,
            writes,
            at,
        });
    }

    // Anchored to the document alone, so a disconnect leaves the cell intact
    // and a change of owner does not move it. One guard per key, whoever wrote
    // it last.
    for (write, name) in named {
        let key = SessionKey {
            prim: write.prim,
            name,
        };
        if find_cell(world, anchor, doc, &key).is_none() {
            world.spawn((
                CellState {
                    doc,
                    key,
                    replicas: replicas.clone(),
                },
                StateDoc(anchor),
            ));
        }
    }
    Ok(())
}

/// A session name is a property name, so it answers to the same key-layout
/// rule every document property does, plus a length the store will accept.
fn valid_name(name: &str) -> Result<PropName, SessionError> {
    if name.len() > replicas::SESSION_NAME_MAX_BYTES {
        return Err(SessionError::BadName);
    }
    name.parse().map_err(|_| SessionError::BadName)
}

/// The live state of a document in the world, or `None` for one this node is
/// not holding open.
fn doc_state(world: &mut World, doc: DocId) -> Option<Arc<Mutex<HsdState>>> {
    world
        .query::<(&HsdDocId, &Hsd)>()
        .iter(world)
        .find(|(id, _)| id.0 == doc)
        .map(|(_, live)| Arc::clone(&live.0))
}

/// Rolls back every session opinion `peer` wrote and composes whatever was
/// underneath back into the documents that held it.
///
/// Answers how many keys it touched. The composition is queued rather than
/// awaited: ejecting a peer is not a frame boundary, and the record is already
/// correct by the time this returns.
#[must_use]
pub fn revert_session(view: &SpaceView, peer: EndpointId) -> usize {
    let restored = view.replicas().revert_writes(peer);
    let touched = restored.len();
    let _ = view
        .commands()
        .push(move |world: &mut World| {
            for one in restored {
                restore_session(world, one);
            }
        })
        .try_send();
    touched
}

fn restore_session(world: &mut World, restored: Restored) {
    let Some(state) = doc_state(world, restored.doc) else {
        return;
    };
    let Ok(mut state) = state.lock() else {
        warn!("scene state poisoned");
        return;
    };

    // The reverted peer's opinion carries the newer stamp, so it is dropped
    // rather than written over: an older write is refused, and rightly.
    state.clear_session(restored.key.prim, &restored.key.name);
    if let Standing::Prior { value, at } = restored.standing {
        let bytes = value.map_or_default(|bytes| Value::Attribute(bytes.into()).encode());
        if let Err(err) = state.apply_session(&Entry::new(
            key::Key::prop(restored.key.prim, &restored.key.name).to_string(),
            bytes,
            at,
        )) {
            warn!(?err, "restored opinion refused by the document");
        }
    }
}

/// Writes the batch into the document's session layer, where it composes with
/// what the document and the scripts say.
///
/// The record in [`Replicas`] is what replication and revert read; this is
/// where the value is drawn from. A document not in the world yet composes
/// nothing — the record carries the opinion until the document arrives and
/// [`Replicas::session_value`] answers for it.
fn compose_session(world: &mut World, doc: DocId, writes: &[(SessionWrite, PropName)], at: u64) {
    let Some(state) = doc_state(world, doc) else {
        return;
    };
    let Ok(mut state) = state.lock() else {
        warn!("scene state poisoned");
        return;
    };

    state.open_tick();
    for (write, name) in writes {
        // A guest states an opaque payload, which is what an attribute is:
        // the same encoding the document uses, so a session opinion on
        // `xform` is interchangeable with the one the document holds.
        let value = write
            .value
            .as_ref()
            .map_or_default(|bytes| Value::Attribute(bytes.clone().into()).encode());
        if let Err(err) = state.apply_session(&Entry::new(
            key::Key::prop(write.prim, name).to_string(),
            value,
            at,
        )) {
            warn!(?err, "session opinion refused by the document");
        }
    }
    state.close_tick();
}

impl SpaceView {
    pub async fn self_pin(&self, space: DocId, doc: DocId) -> bool {
        let me = self.me();
        let at = clock::current_micros();
        self.commands()
            .send_with(move |world: &mut World| {
                let peer_ent = local_peer_entity(world);
                spawn_pin(world, peer_ent, me, doc, space, at, true)
            })
            .await
            .unwrap_or(false)
    }

    pub fn take_hold(&self, space: DocId, doc: DocId) {
        let me = self.me();
        let at = clock::current_micros();
        let _ = self
            .commands()
            .push(move |world: &mut World| {
                let peer_ent = local_peer_entity(world);
                spawn_hold(world, peer_ent, me, doc, space, at, true);
            })
            .try_send();
    }

    /// States what this peer says about `doc`'s prims for the rest of the
    /// session, as one atomic batch.
    ///
    /// A write with no value blocks its key: the property is gone for this
    /// session rather than holding a value.
    /// Drops this peer's hold on `doc`.
    pub fn release_hold(&self, doc: DocId) {
        let _ = self
            .commands()
            .push(move |world: &mut World| {
                if let Some(peer_ent) = entity_by::<LocalPeer, _>(world, |_| true) {
                    clear_hold(world, peer_ent, doc);
                }
            })
            .try_send();
    }

    pub async fn set_session(
        &self,
        space: DocId,
        doc: DocId,
        writes: Vec<SessionWrite>,
    ) -> Result<(), SessionError> {
        let me = self.me();
        let at = clock::current_micros();
        self.commands()
            .send_with(move |world: &mut World| {
                set_session(world, me, doc, space, writes, at, true)
            })
            .await
            .unwrap_or(Err(SessionError::Other))
    }
}

/// Applies a remote peer's delta under `peer_ent`. Called per message from the
/// network recv task; runs in the ECS via [`bevy_async::AsyncCommands`].
pub fn apply_remote(view: &SpaceView, peer_ent: Entity, peer: EndpointId, msg: StateMsg) {
    if view
        .commands()
        .push(move |world: &mut World| apply_in_world(world, peer_ent, peer, msg))
        .try_send()
        .is_err()
    {
        warn!("remote state delta dropped: command queue full");
    }
}

fn apply_in_world(world: &mut World, peer_ent: Entity, peer: EndpointId, msg: StateMsg) {
    match msg {
        StateMsg::Snapshot(snaps) => {
            let existing = world
                .get::<PeerStates>(peer_ent)
                .map_or_default(|s| s.iter().collect::<Vec<_>>());
            for e in existing {
                world.despawn(e);
            }
            for s in snaps {
                if let Some(at) = s.pin.filter(|at| clock::time_valid(*at)) {
                    spawn_pin(world, peer_ent, peer, s.doc, s.space, at, false);
                }
                if let Some(at) = s.hold.filter(|at| clock::time_valid(*at)) {
                    spawn_hold(world, peer_ent, peer, s.doc, s.space, at, false);
                }
                // One batch per stamp, so a snapshot lands the way the writes
                // that made it did.
                let mut by_stamp: BTreeMap<u64, Vec<SessionWrite>> = BTreeMap::new();
                for cell in s.session.into_iter().filter(|c| clock::time_valid(c.at)) {
                    by_stamp.entry(cell.at).or_default().push(SessionWrite {
                        prim:  cell.prim,
                        name:  cell.name,
                        value: cell.value,
                    });
                }
                for (at, writes) in by_stamp {
                    let _ = set_session(world, peer, s.doc, s.space, writes, at, false);
                }
            }
        }
        StateMsg::Pin { doc, space, at } if clock::time_valid(at) => {
            spawn_pin(world, peer_ent, peer, doc, space, at, false);
        }
        StateMsg::Unpin { doc } => clear_pin(world, peer_ent, doc),
        StateMsg::Hold { doc, space, at } if clock::time_valid(at) => {
            spawn_hold(world, peer_ent, peer, doc, space, at, false);
        }
        StateMsg::ReleaseHold { doc } => clear_hold(world, peer_ent, doc),
        StateMsg::Session {
            doc,
            space,
            writes,
            at,
        } if clock::time_valid(at) => {
            let _ = set_session(world, peer, doc, space, writes, at, false);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use hsd::id::PrimId;

    use super::*;

    fn doc(seed: &[u8]) -> DocId {
        DocId(*blake3::hash(seed).as_bytes())
    }

    /// A distinct, valid endpoint id per seed. Arbitrary bytes are not a curve
    /// point, so a key has to be derived rather than written down.
    fn peer(seed: u8) -> EndpointId {
        iroh::SecretKey::from_bytes(&[seed; 32]).public()
    }

    fn prim() -> PrimId {
        PrimId([1; 16])
    }

    /// One opinion on one key, which is all these tests need of a batch.
    fn writes() -> Vec<SessionWrite> {
        vec![SessionWrite {
            prim:  prim(),
            name:  "test/k".to_owned(),
            value: Some(b"v".to_vec()),
        }]
    }

    #[test]
    fn pin_guard_broadcasts_and_releases() {
        let replicas = Replicas::new();
        let policy = Policy::new();
        let me = peer(1);
        let space = doc(b"pin-guard-space");
        let doc = doc(b"pin-guard-doc");

        let (token, rx) = replicas.register_stream(me);
        assert!(matches!(rx.try_recv(), Ok(StateMsg::Snapshot(_))));

        let pin = PinState::register(policy, replicas.clone(), None, me, doc, space, 1, true)
            .expect("pin registers");
        assert_eq!(replicas.owner(space, doc), Some(me));
        assert!(matches!(rx.try_recv(), Ok(StateMsg::Pin { doc: d, .. }) if d == doc));

        drop(pin);
        assert_eq!(replicas.owner(space, doc), None);
        assert!(matches!(rx.try_recv(), Ok(StateMsg::Unpin { doc: d }) if d == doc));

        replicas.unregister_stream(token);
    }

    /// The record is where attribution and replication live; the document's
    /// session layer is where the value composes. One call has to reach both,
    /// or a peer's opinion is replicated and never drawn.
    #[test]
    fn a_session_write_composes_into_the_document_it_speaks_for() {
        let me = peer(1);
        let space = doc(b"compose-space");

        let mut world = World::new();
        world.init_resource::<Policy>();
        world.insert_resource(Replicas::new());
        world.spawn(Space(NamespaceId::from(&space.0)));
        let state = Arc::new(Mutex::new(HsdState::new()));
        world.spawn((Hsd(Arc::clone(&state)), HsdDocId(space)));

        set_session(&mut world, me, space, space, writes(), 1, true).expect("session set");

        let composed = state.lock().expect("lock").get(prim()).and_then(|p| {
            p.property(&hsd::prop_name!("test/k"))
                .and_then(Value::as_attribute)
                .cloned()
        });
        assert_eq!(
            composed.as_deref(),
            Some(&b"v"[..]),
            "what a peer said has to compose in the document it said it about"
        );
    }

    #[test]
    fn a_neutral_cell_guard_drop_does_not_forget() {
        let replicas = Replicas::new();
        let me = peer(1);
        let space = doc(b"neutral-guard-space");

        let (token, rx) = replicas.register_stream(me);
        let _ = rx.try_recv();

        let mut world = World::new();
        world.init_resource::<Policy>();
        world.insert_resource(replicas.clone());
        let space_ent = world.spawn(Space(NamespaceId::from(&space.0))).id();
        set_session(&mut world, me, space, space, writes(), 1, true).expect("session set");
        assert!(matches!(rx.try_recv(), Ok(StateMsg::Session { .. })));

        // Tearing down the doc drops the cell locally but sends no retract, so
        // peers still holding it keep theirs.
        world.despawn(space_ent);
        assert_eq!(
            replicas.session_value(space, space, prim(), &hsd::prop_name!("test/k")),
            None
        );
        assert!(rx.try_recv().is_err());

        replicas.unregister_stream(token);
    }

    #[test]
    fn a_neutral_cell_survives_the_writers_disconnect() {
        let replicas = Replicas::new();
        let remote = peer(3);
        let space = doc(b"neutral-survive-space");

        let mut world = World::new();
        world.init_resource::<Policy>();
        world.insert_resource(replicas.clone());
        let space_ent = world.spawn(Space(NamespaceId::from(&space.0))).id();
        let peer_ent = world.spawn(RemotePeer(remote)).id();
        set_session(&mut world, remote, space, space, writes(), 1, false).expect("session set");
        assert_eq!(
            replicas.session_value(space, space, prim(), &hsd::prop_name!("test/k")),
            Some(b"v".to_vec())
        );

        world.despawn(peer_ent);
        assert_eq!(
            replicas.session_value(space, space, prim(), &hsd::prop_name!("test/k")),
            Some(b"v".to_vec()),
            "a space-owned opinion persists after the peer that wrote it goes"
        );

        // The doc anchor still owns the cell's lifetime.
        world.despawn(space_ent);
        assert_eq!(
            replicas.session_value(space, space, prim(), &hsd::prop_name!("test/k")),
            None
        );
    }

    #[test]
    fn despawning_doc_cascades_state_and_clears_store() {
        let replicas = Replicas::new();
        let policy = Policy::new();
        let me = peer(1);
        let space = doc(b"cascade-doc-space");
        let doc = doc(b"cascade-doc-doc");

        let mut world = World::new();
        world.init_resource::<Policy>();
        world.insert_resource(replicas.clone());
        let peer_ent = world.spawn(LocalPeer).id();
        let doc_ent = world.spawn(SpaceDoc { doc, space }).id();
        let pin = PinState::register(policy, replicas.clone(), None, me, doc, space, 1, false)
            .expect("pin");
        world.spawn((pin, StateDoc(doc_ent), StatePeer(peer_ent)));
        assert_eq!(replicas.owner(space, doc), Some(me));

        world.despawn(doc_ent);
        assert_eq!(replicas.owner(space, doc), None);
        assert!(!replicas.has_doc(space, doc));
    }

    #[test]
    fn despawning_peer_cascades_state_and_clears_store() {
        let replicas = Replicas::new();
        let policy = Policy::new();
        let peer = peer(2);
        let space = doc(b"cascade-peer-space");
        let doc = doc(b"cascade-peer-doc");

        let mut world = World::new();
        world.init_resource::<Policy>();
        world.insert_resource(replicas.clone());
        let peer_ent = world.spawn(RemotePeer(peer)).id();
        let doc_ent = world.spawn(SpaceDoc { doc, space }).id();
        let pin = PinState::register(policy, replicas.clone(), None, peer, doc, space, 1, false)
            .expect("pin");
        world.spawn((pin, StateDoc(doc_ent), StatePeer(peer_ent)));
        assert_eq!(replicas.owner(space, doc), Some(peer));

        world.despawn(peer_ent);
        assert_eq!(replicas.owner(space, doc), None);
        assert!(!replicas.has_doc(space, doc));
    }

    #[test]
    fn the_hold_guard_broadcasts_taking_and_releasing() {
        let replicas = Replicas::new();
        let policy = Policy::new();
        let me = peer(1);
        let space = doc(b"auth-guard-space");
        let doc = doc(b"auth-guard-doc");

        let (token, rx) = replicas.register_stream(me);
        let _ = rx.try_recv();

        let claim = HoldState::register(&policy, &replicas, None, me, doc, space, 5, true)
            .expect("the hold registers");
        assert_eq!(replicas.holder(space, doc), Some(me));
        assert!(matches!(rx.try_recv(), Ok(StateMsg::Hold { doc: d, .. }) if d == doc));

        drop(claim);
        assert_eq!(replicas.holder(space, doc), None);
        assert!(matches!(rx.try_recv(), Ok(StateMsg::ReleaseHold { doc: d }) if d == doc));

        replicas.unregister_stream(token);
    }

    #[test]
    fn superseded_stream_release_keeps_state() {
        let replicas = Replicas::new();
        let peer = peer(2);
        let space = doc(b"supersede-space");
        let doc = doc(b"supersede-doc");

        let mut world = World::new();
        world.init_resource::<Policy>();
        world.insert_resource(replicas.clone());
        let e0 = claim_remote_peer(&mut world, peer, 0);
        assert!(spawn_pin(&mut world, e0, peer, doc, space, 1, false));
        assert_eq!(replicas.owner(space, doc), Some(peer));

        let e1 = claim_remote_peer(&mut world, peer, 1);
        assert_eq!(e0, e1);

        release_remote_peer(&mut world, e0, 0);
        assert_eq!(replicas.owner(space, doc), Some(peer));

        release_remote_peer(&mut world, e1, 1);
        assert_eq!(replicas.owner(space, doc), None);
    }

    #[test]
    fn state_tracked_for_unentered_space() {
        let replicas = Replicas::new();
        let peer = peer(2);
        let space = doc(b"unentered-space");
        let doc = doc(b"unentered-doc");

        // No `Space` entity exists yet.
        let mut world = World::new();
        world.init_resource::<Policy>();
        world.insert_resource(replicas.clone());
        let peer_ent = world.spawn(RemotePeer(peer)).id();
        assert!(spawn_pin(&mut world, peer_ent, peer, doc, space, 1, false));
        assert_eq!(replicas.owner(space, doc), Some(peer));

        let tracker = entity_by::<SpaceDoc, _>(&mut world, |d| d.doc == doc).expect("tracker");
        assert!(world.get::<ChildOf>(tracker).is_none());
        assert_eq!(world.get::<SpaceDoc>(tracker).map(|d| d.space), Some(space));
    }
}
