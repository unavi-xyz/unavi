//! RAII guards tying replicated state to entities, and the world-side apply of
//! local and remote changes.
//!
//! A pin or hold lives exactly as long as its guard entity, and a session
//! layer as long as any of its sources' guards. Guards hang off the document
//! they are about and the peer that stated them, so either despawning takes
//! the state with it.

use std::{
    collections::HashSet,
    sync::Arc,
};

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
use unavi_identity::{
    authorship::{
        self,
        AuthorshipError,
    },
    identity::Identity,
};
use xdid::core::did::Did;

use crate::{
    authority::SpaceView,
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
        frames,
        message::{
            Batch,
            ClearedKey,
            HoldClaim,
            Live,
            MAX_WRITES_PER_BATCH,
            Rejected,
            ReplicationMsg,
            Retained,
            SessionBatch,
            SessionWrite,
        },
        record::{
            PinRecord,
            Record,
        },
        session::{
            SessionError,
            compose::{
                SessionLayerState,
                compose,
                uncompose,
            },
            layers::Applied,
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
        carry_departing(world, peer_ent);
        if let Some(remote) = world.get::<RemotePeer>(peer_ent).map(|remote| remote.id)
            && let Some(replicas) = world.get_resource::<Replicas>()
        {
            replicas.drop_hearsay(remote);
        }
        world.despawn(peer_ent);
    }
}

/// Takes up the session layers a departing peer alone held, when the local
/// rules adopt the rung of the DID each names.
///
/// A carried layer stays its writer's: it is masked by the writer's standing,
/// and this peer relays it in its snapshots. Documents need no carrying, since
/// this peer and every other pinner already hold them.
fn carry_departing(world: &mut World, peer_ent: Entity) {
    let Some(peer) = world.get::<RemotePeer>(peer_ent).map(|remote| remote.id) else {
        return;
    };
    let Some(view) = world.get_resource::<SpaceView>().cloned() else {
        return;
    };
    let replicas = view.replicas();
    let orphans = replicas.orphaned_by(peer, view.me());
    if orphans.is_empty() {
        return;
    }
    let me = Source {
        entity: local_peer_entity(world),
        id:     view.me(),
        did:    Some(view.did().clone()),
        local:  true,
    };
    for orphan in orphans {
        if view.adopts(&orphan.did) && replicas.add_session_source(orphan.writer, orphan.doc, me.id)
        {
            hold_layer(
                world,
                &me,
                orphan.writer,
                &orphan.did,
                orphan.doc,
                orphan.space,
            );
        }
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
    /// A batch its writer sent over its own connection.
    Session(Batch),
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
    /// Layers whose batches are vouched for: the sender's own, or relayed ones
    /// whose signatures verified.
    pub layers:     Vec<Vec<Retained>>,
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

/// Holds `writer`'s layer on `doc` through `holder`'s guard owner, unless a
/// guard for it already lives there.
fn hold_layer(
    world: &mut World,
    holder: &Source,
    writer: EndpointId,
    did: &Did,
    doc: DocId,
    space: SpaceId,
) {
    if find_guard::<SessionLayerState>(world, holder.entity, |layer| {
        layer.writer == writer && layer.doc == doc
    })
    .is_some()
    {
        return;
    }
    let replicas = world.resource::<Replicas>().clone();
    let anchor = doc_anchor(world, doc, space);
    world.spawn((
        SessionLayerState::new(holder.id, writer, did.clone(), doc, replicas),
        ForDoc(anchor),
        FromPeer(holder.entity),
    ));
}

/// Despawns `source`'s guard on `writer`'s layer on `doc`, which drops the
/// layer if it was the last one.
fn drop_source(world: &mut World, source: EndpointId, writer: EndpointId, doc: DocId) {
    let owner = index::lookup::<RemotePeer>(world, source)
        .filter(|e| world.get::<RemotePeer>(*e).is_some())
        .or_else(|| index::lookup::<LocalPeer>(world, ()));
    let Some(owner) = owner else {
        return;
    };
    if let Some(e) = find_guard::<SessionLayerState>(world, owner, |layer| {
        layer.writer == writer && layer.doc == doc
    }) {
        world.despawn(e);
    }
}

/// Applies a batch of session opinions, or the `live` writes of it: records
/// them under `source`, composes them into the document they speak for, and
/// holds the layer for as long as `source`'s guard owner lives.
///
/// One batch, one write boundary. A script's tick flushes its whole dirty set
/// through here, and a half-applied tick can render, so the batch is recorded
/// all or none and composed as one tick.
fn apply_batch(
    world: &mut World,
    source: &Source,
    batch: &Batch,
    live: Option<Live<'_>>,
) -> Result<(), SessionError> {
    let replicas = world.resource::<Replicas>().clone();
    let applied = replicas.add_session(source.id, batch, live)?;
    land_batch(world, source, batch, &applied);
    Ok(())
}

/// Holds the layer a batch landed in and composes what it changed.
fn land_batch(world: &mut World, source: &Source, batch: &Batch, applied: &Applied) {
    let SessionBatch {
        writer,
        did,
        doc,
        space,
        ..
    } = &batch.payload;

    // Spawned before composing, so the new layer is ordered among the other
    // session layers from its first opinion.
    if applied.held {
        hold_layer(world, source, *writer, did, *doc, *space);
    }
    if !applied.cleared.is_empty() {
        uncompose(world, *writer, *doc, &applied.cleared);
    }
    if !applied.set.is_empty() {
        compose(world, *writer, did, *doc, &applied.set);
    }
}

/// Signs `payload` as the local peer's own word, applies it, and broadcasts it.
/// A batch that `restating` the layer's live cells may take it past its batch
/// cap until the older batches drop.
fn write_signed(
    world: &mut World,
    identity: &Identity,
    payload: SessionBatch,
    restating: bool,
) -> Result<(), SessionError> {
    let batch = Batch::sign(payload, identity.signing_key()).map_err(|err| {
        warn!(%err, "session batch not signed");
        SessionError::Unsigned
    })?;
    let me = Source {
        entity: local_peer_entity(world),
        id:     batch.payload.writer,
        did:    Some(batch.payload.did.clone()),
        local:  true,
    };
    let replicas = world.resource::<Replicas>().clone();
    let applied = if restating {
        replicas.add_checkpoint(me.id, &batch)?
    } else {
        replicas.add_session(me.id, &batch, None)?
    };
    land_batch(world, &me, &batch, &applied);
    replicas.broadcast(&ReplicationMsg::Session(batch.signed));
    Ok(())
}

/// Applies and broadcasts the local peer's own batch, then checkpoints its
/// layer if the batches it retains have outgrown its content.
///
/// A batch is one atomic signed message, so one breaking the bounds every
/// receiver checks is refused rather than split.
pub(crate) fn local_write(
    world: &mut World,
    identity: &Identity,
    payload: SessionBatch,
) -> Result<(), SessionError> {
    payload.check().map_err(|err| match err {
        Rejected::NameTooLong => SessionError::BadName,
        _ => SessionError::TooLarge,
    })?;
    let (writer, doc, space) = (payload.writer, payload.doc, payload.space);
    // A layer at the trigger restates itself first, so a writer near its
    // limits has room for the batch.
    checkpoint(world, identity, writer, doc, space);
    write_signed(world, identity, payload, false)?;
    checkpoint(world, identity, writer, doc, space);
    Ok(())
}

/// Re-signs the local peer's live cells on `doc` as fresh batches, so every
/// older batch loses its last cell and is dropped by every holder. A failure
/// is not retried until the layer has grown.
fn checkpoint(world: &mut World, identity: &Identity, me: EndpointId, doc: DocId, space: SpaceId) {
    let replicas = world.resource::<Replicas>().clone();
    if !replicas.needs_checkpoint(me, doc) {
        return;
    }
    let opinions = replicas.session_opinions(me, doc);
    let tombstones = replicas.session_tombstones(me, doc);
    let newest = opinions
        .iter()
        .map(|(.., at)| *at)
        .chain(tombstones.iter().map(|(_, at)| *at))
        .max()
        .unwrap_or(0);
    let at = clock::current_micros().max(newest.saturating_add(1));
    let writes = opinions
        .into_iter()
        .map(|(key, value, _)| SessionWrite {
            prim: key.prim,
            name: key.name.to_string(),
            value,
        })
        .collect();
    let clears = tombstones
        .into_iter()
        .map(|(key, _)| ClearedKey {
            prim: key.prim,
            name: key.name.to_string(),
        })
        .collect::<Vec<_>>();
    let restated = frames::chunk_writes(writes)
        .into_iter()
        .map(|writes| (writes, Vec::new()))
        .chain(
            clears
                .chunks(MAX_WRITES_PER_BATCH)
                .map(|clears| (Vec::new(), clears.to_vec())),
        );
    for (writes, clears) in restated {
        let payload = SessionBatch {
            writer: me,
            did: identity.did().clone(),
            doc,
            space,
            at,
            writes,
            clears,
        };
        if let Err(err) = write_signed(world, identity, payload, true) {
            warn!(%doc, ?err, "session checkpoint failed; retrying once the layer grows");
            replicas.record_checkpoint(me, doc, false);
            return;
        }
    }
    replicas.record_checkpoint(me, doc, true);
}

/// Drops every guard on layers `writer` wrote, whoever holds them. The writer
/// is present, so only its word counts.
fn drop_layers_of(world: &mut World, writer: EndpointId) {
    let replicas = world.resource::<Replicas>().clone();
    for (doc, source) in replicas.session_sources_of_writer(writer) {
        drop_source(world, source, writer, doc);
    }
}

/// Drops every guard on `writer`'s layer on `doc` except `keep`'s.
fn drop_other_sources(world: &mut World, writer: EndpointId, doc: DocId, keep: EndpointId) {
    let replicas = world.resource::<Replicas>().clone();
    for source in replicas.session_sources(writer, doc) {
        if source != keep {
            drop_source(world, source, writer, doc);
        }
    }
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
                    signer: Some(this.clone()),
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

    fn batch(
        &self,
        space: SpaceId,
        doc: DocId,
        writes: Vec<SessionWrite>,
        clears: Vec<ClearedKey>,
    ) -> SessionBatch {
        SessionBatch {
            writer: self.me,
            did: self.identity.identity.did().clone(),
            doc,
            space,
            at: clock::current_micros(),
            writes,
            clears,
        }
    }

    /// Drops this node's own session opinions on `keys` of `doc`, as one
    /// atomic batch.
    pub async fn clear_session(
        &self,
        space: SpaceId,
        doc: DocId,
        keys: Vec<ClearedKey>,
    ) -> Result<(), SessionError> {
        let batch = self.batch(space, doc, Vec::new(), keys);
        let identity = Arc::clone(&self.identity.identity);
        self.async_world
            .commands()
            .send_with(move |world: &mut World| local_write(world, &identity, batch))
            .await
            .unwrap_or(Err(SessionError::Unavailable))
    }

    /// States what this node says about `doc`'s prims for the rest of the
    /// session, as one atomic batch signed with its DID. A write with no value
    /// blocks its key.
    pub async fn set_session(
        &self,
        space: SpaceId,
        doc: DocId,
        writes: Vec<SessionWrite>,
    ) -> Result<(), SessionError> {
        let batch = self.batch(space, doc, writes, Vec::new());
        let identity = Arc::clone(&self.identity.identity);
        self.async_world
            .commands()
            .send_with(move |world: &mut World| local_write(world, &identity, batch))
            .await
            .unwrap_or(Err(SessionError::Unavailable))
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
            drop_layers_of(world, source.id);
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
        Remote::Session(batch) => {
            let SessionBatch { writer, doc, .. } = &batch.payload;
            drop_other_sources(world, *writer, *doc, source.id);
            if let Err(err) = apply_batch(world, &source, &batch, None) {
                debug!(%doc, ?err, "remote session batch refused");
            }
        }
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
    let replicas = world.resource::<Replicas>().clone();
    let me = world.get_resource::<SpaceView>().map(SpaceView::me);
    for layer in doc.layers {
        let Some(first) = layer.first() else {
            continue;
        };
        let writer = first.batch.payload.writer;
        // A relayed layer is ignored while its writer is present, and when it
        // is this peer's own, which a session does not outlive.
        let present = index::lookup::<RemotePeer>(world, writer)
            .is_some_and(|e| world.get::<RemotePeer>(e).is_some());
        let relayed_over_writer = writer != source.id
            && (me == Some(writer)
                || present
                || replicas.session_sources(writer, doc.doc).contains(&writer));
        if relayed_over_writer {
            continue;
        }
        for retained in &layer {
            if let Err(err) = apply_batch(world, source, &retained.batch, Some(retained.live())) {
                debug!(doc = %doc.doc, ?err, "snapshot session batch refused");
            }
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

pub mod pins;
#[cfg(test)] mod tests;
