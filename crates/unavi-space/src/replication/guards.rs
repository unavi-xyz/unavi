//! RAII guards tying replicated state to entities, and the world-side apply of
//! local and remote changes.
//!
//! A pin, hold or cell lives exactly as long as its guard entity. Guards hang
//! off the document they are about and the peer that stated them, so either
//! despawning takes the state with it.

use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        Mutex,
    },
};

use bevy::prelude::*;
use bevy_async::{
    AsyncCommands,
    AsyncWorld,
};
use bevy_hsd::document::{
    DocIndex,
    Hsd,
};
use bevy_iroh::store::DataStore;
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
use unavi_identity::authorship::{
    self,
    Authorship,
};
use xdid::core::did::Did;

use crate::{
    identity::LocalIdentity,
    index::{
        self,
        Index,
        Indexed,
    },
    membership::{
        Space,
        SpaceId,
    },
    pinned::PinnedDoc,
    replication::{
        cell::{
            Restored,
            SessionError,
            SessionKey,
            Standing,
        },
        clock,
        message::{
            ReplicationMsg,
            SESSION_NAME_MAX_BYTES,
            SessionSnapshot,
            SessionWrite,
        },
        store::Replicas,
    },
};

/// Anchors a guard to the document it is about.
#[derive(Component)]
#[relationship(relationship_target = DocGuards)]
pub struct ForDoc(pub Entity);

#[derive(Component, Default)]
#[relationship_target(relationship = ForDoc, linked_spawn)]
pub struct DocGuards(Vec<Entity>);

/// Anchors a guard to the peer that stated it.
#[derive(Component)]
#[relationship(relationship_target = PeerGuards)]
pub struct FromPeer(pub Entity);

#[derive(Component, Default)]
#[relationship_target(relationship = FromPeer, linked_spawn)]
pub struct PeerGuards(Vec<Entity>);

/// The local peer's guard owner, spawned once and living for the session.
/// Local state is cleaned only when its document despawns.
#[derive(Component)]
#[component(on_insert = index::insert::<Self>, on_discard = index::discard::<Self>)]
pub struct LocalPeer;

impl Indexed for LocalPeer {
    type Key = ();

    fn key(&self) {}
}

/// A connected remote peer's guard owner, tied to its state stream.
/// Despawning it on disconnect cascades away all of that peer's state.
#[derive(Component)]
#[component(
    immutable,
    on_insert = index::insert::<Self>,
    on_discard = index::discard::<Self>
)]
pub struct RemotePeer {
    pub id:  EndpointId,
    /// The DID it proved before its link was served.
    pub did: Option<Did>,
}

impl Indexed for RemotePeer {
    type Key = EndpointId;

    fn key(&self) -> EndpointId {
        self.id
    }
}

/// Generation of the state stream feeding a [`RemotePeer`] entity.
///
/// A newer stream (a canonical connection replacing a racing duplicate) takes
/// the entity over, so the superseded stream's teardown leaves it intact.
#[derive(Component)]
pub struct StreamGen(u64);

/// Claims the state entity for `peer`, reusing an existing one so overlapping
/// streams for the same peer never hold duplicate guards.
pub fn claim_remote_peer(
    world: &mut World,
    peer: EndpointId,
    did: Option<Did>,
    generation: u64,
) -> Entity {
    if let Some(e) =
        index::lookup::<RemotePeer>(world, peer).filter(|e| world.get::<RemotePeer>(*e).is_some())
    {
        let newer = world.get::<StreamGen>(e).is_none_or(|g| generation > g.0);
        if newer {
            world.entity_mut(e).insert(StreamGen(generation));
        }
        return e;
    }
    world
        .spawn((RemotePeer { id: peer, did }, StreamGen(generation)))
        .id()
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

#[derive(Component)]
pub struct PinState {
    peer:     EndpointId,
    doc:      DocId,
    local:    bool,
    /// Held because a drop takes no arguments.
    replicas: Replicas,
}

impl Drop for PinState {
    fn drop(&mut self) {
        self.replicas.remove_pin(self.peer, self.doc);
        if self.local {
            self.replicas
                .broadcast(&ReplicationMsg::Unpin { doc: self.doc });
        }
    }
}

#[derive(Component)]
pub struct HoldState {
    peer:     EndpointId,
    doc:      DocId,
    local:    bool,
    /// Set when the release already went out, naming who may take over.
    released: bool,
    /// Held because a drop takes no arguments.
    replicas: Replicas,
}

impl Drop for HoldState {
    fn drop(&mut self) {
        if self.released {
            return;
        }
        self.replicas.remove_hold(self.peer, self.doc, None);
        if self.local {
            self.replicas.broadcast(&ReplicationMsg::ReleaseHold {
                doc: self.doc,
                to:  None,
            });
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
        self.replicas.remove_session(self.doc, &self.key);
    }
}

/// Who stated a change: the guard owner, the endpoint and the DID it proved.
pub struct Stater {
    pub entity: Entity,
    pub id:     EndpointId,
    pub did:    Option<Did>,
    /// Whether this is the local peer, whose changes are broadcast.
    pub local:  bool,
}

/// A remote change whose authorship proofs are already checked.
pub enum Remote {
    Snapshot(Vec<RemoteDoc>),
    Pin {
        doc:   DocId,
        space: SpaceId,
        proof: Proven,
    },
    Unpin {
        doc: DocId,
    },
    Hold {
        doc:   DocId,
        space: SpaceId,
    },
    ReleaseHold {
        doc: DocId,
        to:  Option<EndpointId>,
    },
    Session {
        doc:    DocId,
        space:  SpaceId,
        writes: Vec<SessionWrite>,
        at:     u64,
    },
}

/// One document of a remote snapshot.
pub struct RemoteDoc {
    pub doc:     DocId,
    pub space:   SpaceId,
    pub pin:     Option<Proven>,
    pub hold:    bool,
    pub session: Vec<SessionSnapshot>,
}

/// An authorship proof whose namespace signature checked out and whose DID is
/// the one its sender proved.
pub struct Proven {
    pub author: Did,
    pub proof:  Authorship,
}

fn space_entity(world: &World, space: SpaceId) -> Option<Entity> {
    index::lookup::<Space>(world, space)
}

/// Resolves the entity anchoring `doc`, spawning a [`PinnedDoc`] tracker when
/// nothing exists yet. Trackers for spaces not yet entered stay unparented so
/// their state is kept until the space is joined.
fn doc_anchor(world: &mut World, doc: DocId, space: SpaceId) -> Entity {
    if let Some(e) = space_entity(world, SpaceId::of_doc(doc)) {
        return e;
    }
    if let Some(e) = world.get_resource::<DocIndex>().and_then(|i| i.get(doc)) {
        return e;
    }
    if let Some(e) = index::lookup::<PinnedDoc>(world, doc) {
        return e;
    }
    let space_ent = space_entity(world, space);
    let mut tracker = world.spawn(PinnedDoc { doc, space });
    if let Some(space_ent) = space_ent {
        tracker.insert(ChildOf(space_ent));
    }
    tracker.id()
}

fn local_peer_entity(world: &mut World) -> Entity {
    if let Some(e) = index::lookup::<LocalPeer>(world, ()) {
        return e;
    }
    world.spawn(LocalPeer).id()
}

/// Finds a guard of `C` owned by `peer_ent` matching `pred`.
fn find_guard<C: Component>(
    world: &World,
    peer_ent: Entity,
    pred: impl Fn(&C) -> bool,
) -> Option<Entity> {
    world
        .get::<PeerGuards>(peer_ent)?
        .iter()
        .find(|e| world.get::<C>(*e).is_some_and(&pred))
}

/// Finds the cell guard for `key` on `anchor`, if one exists.
fn find_cell(world: &World, anchor: Entity, doc: DocId, key: &SessionKey) -> Option<Entity> {
    world.get::<DocGuards>(anchor)?.iter().find(|e| {
        world
            .get::<CellState>(*e)
            .is_some_and(|c| c.doc == doc && &c.key == key)
    })
}

fn spawn_pin(
    world: &mut World,
    stater: &Stater,
    doc: DocId,
    space: SpaceId,
    proven: Proven,
) -> bool {
    if find_guard::<PinState>(world, stater.entity, |p| p.doc == doc).is_some() {
        return true;
    }
    let replicas = world.resource::<Replicas>().clone();
    let msg = stater.local.then(|| ReplicationMsg::Pin {
        doc,
        space,
        author: proven.proof.clone(),
    });
    let at = clock::current_micros();
    if let Err(err) = replicas.add_pin(stater.id, doc, space, proven.author, proven.proof, at) {
        warn!(%doc, %err, "pin refused");
        return false;
    }
    if let Some(msg) = msg {
        replicas.broadcast(&msg);
    }
    let anchor = doc_anchor(world, doc, space);
    world.spawn((
        PinState {
            peer: stater.id,
            doc,
            local: stater.local,
            replicas,
        },
        ForDoc(anchor),
        FromPeer(stater.entity),
    ));
    true
}

fn spawn_hold(world: &mut World, stater: &Stater, doc: DocId, space: SpaceId) -> bool {
    let replicas = world.resource::<Replicas>().clone();
    let at = clock::current_micros();
    if !replicas.add_hold(stater.id, stater.did.as_ref(), doc, space, at) {
        debug!(%doc, peer = %stater.id, "hold refused");
        return false;
    }
    if stater.local {
        replicas.broadcast(&ReplicationMsg::Hold { doc, space });
    }
    if find_guard::<HoldState>(world, stater.entity, |h| h.doc == doc).is_none() {
        let anchor = doc_anchor(world, doc, space);
        world.spawn((
            HoldState {
                peer: stater.id,
                doc,
                local: stater.local,
                released: false,
                replicas,
            },
            ForDoc(anchor),
            FromPeer(stater.entity),
        ));
    }
    true
}

fn release_hold(world: &mut World, stater: &Stater, doc: DocId, to: Option<EndpointId>) {
    let replicas = world.resource::<Replicas>().clone();
    replicas.remove_hold(stater.id, doc, to);
    if stater.local {
        replicas.broadcast(&ReplicationMsg::ReleaseHold { doc, to });
    }
    if let Some(e) = find_guard::<HoldState>(world, stater.entity, |h| h.doc == doc) {
        if let Some(mut hold) = world.get_mut::<HoldState>(e) {
            hold.released = true;
        }
        world.despawn(e);
    }
}

fn clear_pin(world: &mut World, peer_ent: Entity, doc: DocId) {
    if let Some(e) = find_guard::<PinState>(world, peer_ent, |p| p.doc == doc) {
        world.despawn(e);
    }
}

/// Applies a batch of session opinions: records them, composes them into the
/// document they speak for, and broadcasts them if they are this peer's.
///
/// One batch, one write boundary. A script's tick flushes its whole dirty set
/// through here, and a half-applied tick can render, so the batch is recorded
/// all or none and composed as one tick.
fn set_session(
    world: &mut World,
    stater: &Stater,
    doc: DocId,
    space: SpaceId,
    writes: Vec<SessionWrite>,
    at: u64,
) -> Result<(), SessionError> {
    let named: Vec<(SessionWrite, PropName)> = writes
        .into_iter()
        .map(|w| valid_name(&w.name).map(|name| (w, name)))
        .collect::<Result<_, _>>()?;

    let replicas = world.resource::<Replicas>().clone();
    replicas.add_session(
        stater.id,
        stater.did.as_ref(),
        doc,
        space,
        named
            .iter()
            .map(|(w, name)| {
                (
                    SessionKey {
                        prim: w.prim,
                        name: name.clone(),
                    },
                    w.value.clone(),
                )
            })
            .collect(),
        at,
    )?;

    compose_session(world, doc, &named, at);

    if stater.local {
        let writes = named.iter().map(|(w, _)| w.clone()).collect();
        replicas.broadcast(&ReplicationMsg::Session {
            doc,
            space,
            writes,
            at,
        });
    }

    // Anchored to the document alone, so a disconnect leaves the cell intact.
    // One guard per key, whoever wrote it last.
    let anchor = doc_anchor(world, doc, space);
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
                ForDoc(anchor),
            ));
        }
    }
    Ok(())
}

/// A session name is a property name, so it answers to the same key-layout
/// rule every document property does, plus a length the store will accept.
fn valid_name(name: &str) -> Result<PropName, SessionError> {
    if name.len() > SESSION_NAME_MAX_BYTES {
        return Err(SessionError::BadName);
    }
    name.parse().map_err(|_| SessionError::BadName)
}

/// The live state of a document in the world, or `None` for one this node is
/// not holding open.
fn doc_state(world: &World, doc: DocId) -> Option<Arc<Mutex<HsdState>>> {
    let entity = world.get_resource::<DocIndex>()?.get(doc)?;
    world.get::<Hsd>(entity).map(|live| Arc::clone(&live.0))
}

/// Rolls back every session opinion `peer` wrote and composes whatever was
/// underneath back into the documents that held it.
///
/// Answers how many keys it touched. The composition is queued rather than
/// awaited: the record is already correct by the time this returns.
#[must_use]
pub fn revert_session(replicas: &Replicas, commands: AsyncCommands, peer: EndpointId) -> usize {
    let restored = replicas.revert_writes(peer);
    let touched = restored.len();
    let _ = commands
        .push(move |world: &mut World| {
            for one in restored {
                restore_session(world, one);
            }
        })
        .try_send();
    touched
}

fn restore_session(world: &World, restored: Restored) {
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
/// A document not in the world yet composes nothing; the record carries the
/// opinion until the document arrives and [`Replicas::session_value`] answers
/// for it.
fn compose_session(world: &World, doc: DocId, writes: &[(SessionWrite, PropName)], at: u64) {
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

/// Why this node could not pin a document.
#[derive(Debug, thiserror::Error)]
pub enum PinError {
    #[error("no local store")]
    NoStore,
    #[error("the world is gone")]
    Unavailable,
    #[error("this node cannot write the document")]
    NotAuthor(#[from] unavi_identity::authorship::AuthorshipError),
    #[error("the pin was refused")]
    Refused,
}

/// This node's own pins, holds and session writes. Each one is applied
/// locally under the same rules a peer's is, then broadcast.
#[derive(Clone)]
pub struct LocalReplica {
    me:          EndpointId,
    identity:    LocalIdentity,
    async_world: AsyncWorld,
}

impl LocalReplica {
    #[must_use]
    pub const fn new(me: EndpointId, identity: LocalIdentity, async_world: AsyncWorld) -> Self {
        Self {
            me,
            identity,
            async_world,
        }
    }

    fn stater(&self, world: &mut World) -> Stater {
        Stater {
            entity: local_peer_entity(world),
            id:     self.me,
            did:    Some(self.identity.identity.did().clone()),
            local:  true,
        }
    }

    /// Pins `doc` into `space`, recording this node as its author. Only a
    /// document this node can write may be pinned.
    pub async fn pin(&self, space: SpaceId, doc: DocId) -> Result<(), PinError> {
        let store = self
            .async_world
            .commands()
            .send_with(|world: &mut World| world.get_resource::<DataStore>().map(|s| s.0.clone()))
            .await
            .ok_or(PinError::Unavailable)?
            .ok_or(PinError::NoStore)?;
        let ns = iroh_docs::NamespaceId::from(&doc.0);
        let held = store
            .held(ns)
            .await
            .map_err(|err| PinError::NotAuthor(err.into()))?
            .ok_or(PinError::NoStore)?;
        let proof = authorship::claim(&store, &held, &self.identity.identity).await?;

        let this = self.clone();
        let pinned = self
            .async_world
            .commands()
            .send_with(move |world: &mut World| {
                let stater = this.stater(world);
                let proven = Proven {
                    author: this.identity.identity.did().clone(),
                    proof,
                };
                spawn_pin(world, &stater, doc, space, proven)
            })
            .await
            .ok_or(PinError::Unavailable)?;
        if pinned {
            Ok(())
        } else {
            Err(PinError::Refused)
        }
    }

    /// Takes hold of `doc`, if this node authors it or was released to.
    pub fn take_hold(&self, space: SpaceId, doc: DocId) {
        let this = self.clone();
        let _ = self
            .async_world
            .commands()
            .push(move |world: &mut World| {
                let stater = this.stater(world);
                spawn_hold(world, &stater, doc, space);
            })
            .try_send();
    }

    /// Drops this node's hold on `doc`. With `to`, that peer may take hold
    /// next.
    pub fn release_hold(&self, doc: DocId, to: Option<EndpointId>) {
        let this = self.clone();
        let _ = self
            .async_world
            .commands()
            .push(move |world: &mut World| {
                let stater = this.stater(world);
                release_hold(world, &stater, doc, to);
            })
            .try_send();
    }

    /// States what this node says about `doc`'s prims for the rest of the
    /// session, as one atomic batch. A write with no value blocks its key.
    pub async fn set_session(
        &self,
        space: SpaceId,
        doc: DocId,
        writes: Vec<SessionWrite>,
    ) -> Result<(), SessionError> {
        let at = clock::current_micros();
        let this = self.clone();
        self.async_world
            .commands()
            .send_with(move |world: &mut World| {
                let stater = this.stater(world);
                set_session(world, &stater, doc, space, writes, at)
            })
            .await
            .unwrap_or(Err(SessionError::Unavailable))
    }
}

/// Applies a remote peer's change under its [`RemotePeer`] entity.
pub fn apply_remote(world: &mut World, peer_ent: Entity, change: Remote) {
    let Some(stater) = world.get::<RemotePeer>(peer_ent).map(|r| Stater {
        entity: peer_ent,
        id:     r.id,
        did:    r.did.clone(),
        local:  false,
    }) else {
        return;
    };

    match change {
        Remote::Snapshot(docs) => {
            let existing = world
                .get::<PeerGuards>(peer_ent)
                .map_or_default(|s| s.iter().collect::<Vec<_>>());
            for e in existing {
                world.despawn(e);
            }
            for doc in docs {
                apply_snapshot_doc(world, &stater, doc);
            }
        }
        Remote::Pin { doc, space, proof } => {
            spawn_pin(world, &stater, doc, space, proof);
        }
        Remote::Unpin { doc } => clear_pin(world, peer_ent, doc),
        Remote::Hold { doc, space } => {
            spawn_hold(world, &stater, doc, space);
        }
        Remote::ReleaseHold { doc, to } => release_hold(world, &stater, doc, to),
        Remote::Session {
            doc,
            space,
            writes,
            at,
        } => {
            if let Err(err) = set_session(world, &stater, doc, space, writes, at) {
                debug!(%doc, ?err, "remote session batch refused");
            }
        }
    }
}

fn apply_snapshot_doc(world: &mut World, stater: &Stater, doc: RemoteDoc) {
    if let Some(proof) = doc.pin {
        spawn_pin(world, stater, doc.doc, doc.space, proof);
    }
    if doc.hold {
        spawn_hold(world, stater, doc.doc, doc.space);
    }
    // One batch per stamp, so a snapshot lands the way the writes that made
    // it did.
    let mut by_stamp: BTreeMap<u64, Vec<SessionWrite>> = BTreeMap::new();
    for cell in doc.session {
        by_stamp.entry(cell.at).or_default().push(SessionWrite {
            prim:  cell.prim,
            name:  cell.name,
            value: cell.value,
        });
    }
    for (at, writes) in by_stamp {
        if let Err(err) = set_session(world, stater, doc.doc, doc.space, writes, at) {
            debug!(doc = %doc.doc, ?err, "snapshot session batch refused");
        }
    }
}

/// Registers the indexes guards resolve through.
pub fn init_indexes(world: &mut World) {
    world.init_resource::<Index<Space>>();
    world.init_resource::<Index<PinnedDoc>>();
    world.init_resource::<Index<LocalPeer>>();
    world.init_resource::<Index<RemotePeer>>();
    world.init_resource::<DocIndex>();
}

#[cfg(test)] mod tests;
