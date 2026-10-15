//! RAII guards tying replicated state to entities, and the world-side apply of
//! local and remote changes.
//!
//! A pin or hold lives exactly as long as its guard entity, and a session
//! layer as long as any of its sources' guards. Guards hang off the document
//! they are about and the peer that stated them, so either despawning takes
//! the state with it.

use std::collections::HashSet;

use bevy::prelude::*;
use bevy_async::{
    AsyncWorld,
    task,
};
use bevy_hsd::document::DocIndex;
use bevy_iroh::store::DataStore;
use hsd::id::DocId;
use iroh::EndpointId;
use pins::{
    PinClaim,
    PinState,
    clear_pin,
    note_record,
    spawn_pin,
};
use unavi_identity::authorship::{
    self,
    AuthorshipError,
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
        clock,
        message::{
            HoldClaim,
            ReplicationMsg,
            session::{
                ReceivedLayer,
                ReceivedTick,
            },
        },
        record::{
            PinRecord,
            Record,
        },
        store::{
            Proven,
            Replicas,
        },
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
        // The newer stream's connection proved the DID changes are judged by.
        if newer {
            world
                .entity_mut(e)
                .insert((RemotePeer { id: peer, did }, StreamGen(generation)));
        }
        return e;
    }
    world
        .spawn((RemotePeer { id: peer, did }, StreamGen(generation)))
        .id()
}

/// Despawns the peer's state entity only if `generation` still owns it,
/// carrying what it leaves behind first.
pub fn release_remote_peer(world: &mut World, peer_ent: Entity, generation: u64) {
    if world
        .get::<StreamGen>(peer_ent)
        .is_some_and(|g| g.0 == generation)
    {
        if let Some(remote) = world.get::<RemotePeer>(peer_ent).map(|remote| remote.id) {
            layers::carry_departing(world, remote);
            if let Some(replicas) = world.get_resource::<Replicas>() {
                replicas.drop_hearsay(remote);
                replicas.forget_refused(remote);
            }
        }
        world.despawn(peer_ent);
    }
}

/// A peer's hold on a document, released when the guard drops.
#[derive(Component)]
pub struct HoldState {
    peer:     EndpointId,
    doc:      DocId,
    local:    bool,
    /// Held because a drop takes no arguments.
    replicas: Replicas,
}

impl Drop for HoldState {
    fn drop(&mut self) {
        self.replicas.remove_hold(self.peer, self.doc);
        if self.local {
            self.replicas
                .broadcast(&ReplicationMsg::ReleaseHold { doc: self.doc });
        }
    }
}

/// The peer a change came from: its guard owner, endpoint and proven DID.
pub struct Source {
    pub entity: Entity,
    pub id:     EndpointId,
    pub did:    Option<Did>,
    /// Whether this is the local peer, whose changes are broadcast.
    pub local:  bool,
}

/// A remote change whose proofs and records are already checked.
pub enum Remote {
    Snapshot(Vec<RemoteDoc>),
    Pin(PinClaim),
    Unpin {
        doc:        DocId,
        withdrawal: Option<Record>,
    },
    Hold(HoldClaim),
    ReleaseHold {
        doc: DocId,
    },
    /// A tick its writer sent over its own connection.
    Session(ReceivedTick),
}

/// One document of a remote snapshot.
pub struct RemoteDoc {
    pub doc:        DocId,
    pub space:      SpaceId,
    /// Present only with a record saying the document is pinned.
    pub pin:        Option<PinClaim>,
    /// The stamp of the sender's hold, within the skew allowance.
    pub hold:       Option<u64>,
    /// The newest withdrawal the sender knows.
    pub withdrawal: Option<Record>,
    /// Layers vouched for: the sender's own, or relayed ones whose signature
    /// and root verified.
    pub layers:     Vec<ReceivedLayer>,
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

/// Records or restates `source`'s hold.
fn spawn_hold(world: &mut World, source: &Source, hold: HoldClaim) {
    let HoldClaim { doc, space, .. } = hold;
    let replicas = world.resource::<Replicas>().clone();
    if !replicas.add_hold(source.id, hold) {
        debug!(%doc, peer = %source.id, "hold refused: the peer holds too many documents");
        return;
    }
    if source.local {
        replicas.broadcast(&ReplicationMsg::Hold(hold));
    }
    if find_guard::<HoldState>(world, source.entity, |h| h.doc == doc).is_none() {
        let anchor = doc_anchor(world, doc, space);
        world.spawn((
            HoldState {
                peer: source.id,
                doc,
                local: source.local,
                replicas,
            },
            ForDoc(anchor),
            FromPeer(source.entity),
        ));
    }
}

/// Drops `source`'s hold on `doc`. The guard's drop releases it.
fn release_hold(world: &mut World, source: &Source, doc: DocId) {
    if let Some(e) = find_guard::<HoldState>(world, source.entity, |h| h.doc == doc) {
        world.despawn(e);
    }
}

/// Why this node could not pin or withdraw a document.
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
    #[error("the pin record could not be signed")]
    Unsigned,
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

    fn source(&self, world: &mut World) -> Source {
        Source {
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
        let key = store
            .write_key(ns)
            .await
            .map_err(|err| PinError::NotAuthor(err.into()))?
            .ok_or(PinError::NotAuthor(AuthorshipError::NotWritable))?;

        let this = self.clone();
        self.async_world
            .commands()
            .send_with(move |world: &mut World| {
                let source = this.source(world);
                let floor = world.resource::<Replicas>().newest_record_at(doc);
                let at = clock::current_micros().max(floor.map_or(0, |at| at.saturating_add(1)));
                let payload = PinRecord {
                    doc,
                    space,
                    at,
                    pinned: true,
                };
                let record = match Record::sign(&key, payload) {
                    Ok(record) => record,
                    Err(err) => {
                        warn!(%doc, %err, "pin record not signed");
                        return Err(PinError::Unsigned);
                    }
                };
                let claim = PinClaim {
                    proven: Proven {
                        author: this.identity.identity.did().clone(),
                        proof,
                    },
                    record,
                };
                if spawn_pin(world, &source, claim) {
                    Ok(())
                } else {
                    Err(PinError::Refused)
                }
            })
            .await
            .ok_or(PinError::Unavailable)?
    }

    /// Takes hold of `doc`, outranking every hold known on it.
    pub fn take_hold(&self, space: SpaceId, doc: DocId) {
        let this = self.clone();
        self.queue(move |world: &mut World| {
            let source = this.source(world);
            let newest = world.resource::<Replicas>().newest_hold_at(doc);
            let at = clock::stamp_after(newest, clock::current_micros());
            spawn_hold(world, &source, HoldClaim { doc, space, at });
        });
    }

    /// Drops this node's hold on `doc`.
    pub fn release_hold(&self, doc: DocId) {
        let this = self.clone();
        self.queue(move |world: &mut World| {
            let source = this.source(world);
            release_hold(world, &source, doc);
        });
    }

    /// Runs `f` on the world, waiting for room when its queue is full.
    fn queue(&self, f: impl FnOnce(&mut World) + Clone + Send + 'static) {
        if self
            .async_world
            .commands()
            .push(f.clone())
            .try_send()
            .is_ok()
        {
            return;
        }
        let commands = self.async_world.commands();
        task::spawn(async move {
            if commands.push(f).send().await.is_err() {
                debug!("the world is gone; local replica change dropped");
            }
        });
    }
}

/// Applies a remote peer's change under its [`RemotePeer`] entity.
pub fn apply_remote(world: &mut World, peer_ent: Entity, change: Remote) {
    let Some(source) = world.get::<RemotePeer>(peer_ent).map(|r| Source {
        entity: peer_ent,
        id:     r.id,
        did:    r.did.clone(),
        local:  false,
    }) else {
        return;
    };

    match change {
        Remote::Snapshot(docs) => {
            drop_unstated(world, peer_ent, &docs);
            layers::drop_layers_of(world, source.id);
            for doc in docs {
                apply_snapshot_doc(world, &source, doc);
            }
        }
        Remote::Pin(claim) => {
            spawn_pin(world, &source, claim);
        }
        Remote::Unpin { doc, withdrawal } => {
            clear_pin(world, peer_ent, doc);
            if let Some(record) = withdrawal {
                note_record(world, &record, source.id);
            }
        }
        Remote::Hold(hold) => spawn_hold(world, &source, hold),
        Remote::ReleaseHold { doc } => release_hold(world, &source, doc),
        Remote::Session(tick) => layers::apply_tick(world, &source, &tick),
    }
}

/// Despawns the guards of a peer's that its snapshot no longer states. Pins
/// and holds it restates keep their guards, so nothing it still states is
/// released and taken again. Every layer guard goes, since a snapshot
/// replaces the layers it held.
fn drop_unstated(world: &mut World, peer_ent: Entity, docs: &[RemoteDoc]) {
    let pinned = docs
        .iter()
        .filter(|doc| doc.pin.is_some())
        .map(|doc| doc.doc)
        .collect::<HashSet<_>>();
    let held = docs
        .iter()
        .filter(|doc| doc.hold.is_some())
        .map(|doc| doc.doc)
        .collect::<HashSet<_>>();
    let unstated = world.get::<PeerGuards>(peer_ent).map_or_default(|guards| {
        guards
            .iter()
            .filter(|e| {
                let pin_doc = world.get::<PinState>(*e).map(|pin| pin.doc);
                let hold_doc = world.get::<HoldState>(*e).map(|hold| hold.doc);
                !pin_doc.is_some_and(|doc| pinned.contains(&doc))
                    && !hold_doc.is_some_and(|doc| held.contains(&doc))
            })
            .collect::<Vec<_>>()
    });
    for e in unstated {
        world.despawn(e);
    }
}

fn apply_snapshot_doc(world: &mut World, source: &Source, doc: RemoteDoc) {
    if let Some(record) = &doc.withdrawal {
        note_record(world, record, source.id);
    }
    if let Some(claim) = doc.pin {
        spawn_pin(world, source, claim);
    }
    if let Some(at) = doc.hold {
        spawn_hold(
            world,
            source,
            HoldClaim {
                doc: doc.doc,
                space: doc.space,
                at,
            },
        );
    }
    for layer in doc.layers {
        layers::apply_layer(world, source, layer);
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

pub mod layers;
pub mod pins;
#[cfg(test)] mod tests;
