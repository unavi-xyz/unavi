//! Session layer guards, and the ticks and whole layers that fill them.

use std::sync::Arc;

use bevy::prelude::*;
use hsd::id::DocId;
use iroh::EndpointId;
use unavi_identity::identity::Identity;

use crate::{
    authority::SpaceView,
    index,
    membership::SpaceId,
    replication::{
        guards::{
            ForDoc,
            FromPeer,
            LocalPeer,
            LocalReplica,
            RemotePeer,
            Source,
            doc_anchor,
            find_guard,
            local_peer_entity,
        },
        message::{
            Rejected,
            ReplicationMsg,
            session::{
                ClearedKey,
                LayerState,
                ReceivedLayer,
                ReceivedTick,
                SessionTick,
                SessionWrite,
                SignedLayer,
                check_tick,
            },
        },
        session::{
            SessionError,
            WriterId,
            compose::{
                SessionLayerState,
                compose,
                uncompose,
            },
            layers::{
                Applied,
                LocalTick,
            },
        },
        store::Replicas,
    },
};

/// The local writer's own opinions on one document, sent as one tick.
pub(crate) struct LocalWrite {
    pub(crate) doc:    DocId,
    pub(crate) space:  SpaceId,
    pub(crate) writes: Vec<SessionWrite>,
    pub(crate) clears: Vec<ClearedKey>,
}

/// Holds `writer`'s layer on `doc` through `holder`'s guard owner, unless a
/// guard for it already lives there.
fn hold_layer(world: &mut World, holder: &Source, state: &LayerState) {
    let writer = WriterId::new(state.device, &state.did);
    if find_guard::<SessionLayerState>(world, holder.entity, |layer| {
        layer.writer == writer && layer.doc == state.doc
    })
    .is_some()
    {
        return;
    }
    let replicas = world.resource::<Replicas>().clone();
    let anchor = doc_anchor(world, state.doc, state.space);
    world.spawn((
        SessionLayerState::new(holder.id, writer, state.did.clone(), state.doc, replicas),
        ForDoc(anchor),
        FromPeer(holder.entity),
    ));
}

/// Despawns `source`'s guard on `writer`'s layer on `doc`, which drops the
/// layer if it was the last one.
fn drop_source(world: &mut World, source: EndpointId, writer: WriterId, doc: DocId) {
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

/// Holds the layer a change landed in, through `source`, and composes what it
/// changed as one tick.
fn land(world: &mut World, source: &Source, state: &LayerState, applied: &Applied) {
    let writer = WriterId::new(state.device, &state.did);
    // Spawned before composing, so the new layer is ordered among the other
    // session layers from its first opinion.
    if applied.held {
        hold_layer(world, source, state);
    }
    if !applied.cleared.is_empty() {
        uncompose(world, writer, state.doc, &applied.cleared);
    }
    if !applied.set.is_empty() {
        compose(world, writer, &state.did, state.doc, &applied.set);
    }
}

/// Applies a tick its writer sent over its own link. The writer is present,
/// so every other peer's copy of its layer gives way to this one.
pub(super) fn apply_tick(world: &mut World, source: &Source, tick: &ReceivedTick) {
    let state = &tick.state.payload;
    drop_other_sources(world, tick.state.writer(), state.doc, source.id);
    let replicas = world.resource::<Replicas>().clone();
    match replicas.apply_tick(source.id, tick) {
        Ok(applied) => land(world, source, state, &applied),
        Err(err) => debug!(doc = %state.doc, %err, "session tick refused"),
    }
}

/// Applies a whole layer a snapshot states, unless its writer is present or
/// is this peer, whose word on its own layer counts over any relay.
pub(super) fn apply_layer(world: &mut World, source: &Source, layer: ReceivedLayer) {
    let writer = layer.state.writer();
    let doc = layer.state.payload.doc;
    let replicas = world.resource::<Replicas>().clone();
    if writer.device != source.id {
        let me = world.get_resource::<SpaceView>().map(SpaceView::me);
        let present = index::lookup::<RemotePeer>(world, writer.device)
            .is_some_and(|e| world.get::<RemotePeer>(e).is_some());
        if me == Some(writer.device)
            || present
            || replicas
                .session_sources(writer, doc)
                .contains(&writer.device)
        {
            return;
        }
    }
    let state = layer.state.payload.clone();
    match replicas.replace_layer(source.id, layer) {
        Ok(applied) => land(world, source, &state, &applied),
        Err(err) => debug!(%doc, %err, "snapshot session layer refused"),
    }
}

/// Applies, signs and broadcasts the local writer's own tick, refused whole
/// when it breaks the bounds every receiver checks.
pub(crate) fn local_write(
    world: &mut World,
    identity: &Identity,
    me: EndpointId,
    write: LocalWrite,
) -> Result<(), SessionError> {
    let (writes, clears) = check_tick(&write.writes, &write.clears).map_err(|err| match err {
        Rejected::NameTooLong | Rejected::Malformed => SessionError::BadName,
        _ => SessionError::TooLarge,
    })?;
    let tick = LocalTick {
        device: me,
        did:    identity.did(),
        doc:    write.doc,
        space:  write.space,
        writes: &writes,
        clears: &clears,
    };
    let replicas = world.resource::<Replicas>().clone();
    let (applied, state) = replicas.write_local(&tick, |payload| {
        SignedLayer::sign(payload, identity.signing_key())
    })?;
    let source = Source {
        entity: local_peer_entity(world),
        id:     me,
        did:    Some(identity.did().clone()),
        local:  true,
    };
    land(world, &source, &state.payload, &applied);
    replicas.broadcast(&ReplicationMsg::Session(SessionTick {
        writes: write.writes,
        clears: write.clears,
        state:  state.signed,
    }));
    Ok(())
}

/// Drops every guard on layers `device` writes, whoever holds them. The device
/// is present, so only its word counts.
pub(super) fn drop_layers_of(world: &mut World, device: EndpointId) {
    let replicas = world.resource::<Replicas>().clone();
    for (doc, writer, source) in replicas.session_sources_of_device(device) {
        drop_source(world, source, writer, doc);
    }
}

/// Drops every guard on `writer`'s layer on `doc` except `keep`'s.
fn drop_other_sources(world: &mut World, writer: WriterId, doc: DocId, keep: EndpointId) {
    let replicas = world.resource::<Replicas>().clone();
    for source in replicas.session_sources(writer, doc) {
        if source != keep {
            drop_source(world, source, writer, doc);
        }
    }
}

/// Takes up the session layers a departing peer alone held, when the local
/// rules adopt the rung of the DID each names.
///
/// A carried layer stays its writer's: it is masked by the writer's standing,
/// and this peer relays it in its snapshots.
pub(super) fn carry_departing(world: &mut World, departing: EndpointId) {
    let Some(view) = world.get_resource::<SpaceView>().cloned() else {
        return;
    };
    let replicas = view.replicas();
    let orphans = replicas.orphaned_by(departing, view.me());
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
            let writer = SessionLayerState::new(
                me.id,
                orphan.writer,
                orphan.did.clone(),
                orphan.doc,
                replicas.clone(),
            );
            let anchor = doc_anchor(world, orphan.doc, orphan.space);
            world.spawn((writer, ForDoc(anchor), FromPeer(me.entity)));
        }
    }
}

impl LocalReplica {
    /// States what this node says about `doc`'s prims for the rest of the
    /// session, as one tick. A write with no value blocks its key.
    pub async fn set_session(
        &self,
        space: SpaceId,
        doc: DocId,
        writes: Vec<SessionWrite>,
    ) -> Result<(), SessionError> {
        self.write(LocalWrite {
            doc,
            space,
            writes,
            clears: Vec::new(),
        })
        .await
    }

    /// Drops this node's own session opinions on `keys` of `doc`, as one tick.
    pub async fn clear_session(
        &self,
        space: SpaceId,
        doc: DocId,
        keys: Vec<ClearedKey>,
    ) -> Result<(), SessionError> {
        self.write(LocalWrite {
            doc,
            space,
            writes: Vec::new(),
            clears: keys,
        })
        .await
    }

    async fn write(&self, write: LocalWrite) -> Result<(), SessionError> {
        let identity = Arc::clone(&self.identity.identity);
        let me = self.me;
        self.async_world
            .commands()
            .send_with(move |world: &mut World| local_write(world, &identity, me, write))
            .await
            .unwrap_or(Err(SessionError::Unavailable))
    }
}
