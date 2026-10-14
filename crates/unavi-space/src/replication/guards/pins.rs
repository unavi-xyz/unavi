//! Pin guards, and the pin records and withdrawals they state.

use bevy::prelude::*;
use bevy_iroh::store::DataStore;
use hsd::id::DocId;
use iroh::EndpointId;
use iroh_docs::{
    NamespaceId,
    NamespaceSecret,
};
use unavi_identity::authorship::AuthorshipError;

use crate::{
    authority::SpaceView,
    index,
    replication::{
        clock,
        guards::{
            ForDoc,
            FromPeer,
            LocalPeer,
            LocalReplica,
            PeerGuards,
            PinError,
            Source,
            doc_anchor,
            find_guard,
            local_peer_entity,
        },
        message::ReplicationMsg,
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

/// A peer's pin on a document, unpinned when the guard drops.
#[derive(Component)]
pub struct PinState {
    peer:           EndpointId,
    pub(super) doc: DocId,
    local:          bool,
    /// Whether the pinner proved the document's author.
    by_author:      bool,
    /// The withdrawal the unpin carries: the author's own when it withdraws
    /// the document, or the one a non-author pin relays when it drops.
    withdrawal:     Option<Record>,
    /// Held because a drop takes no arguments.
    replicas:       Replicas,
}

impl Drop for PinState {
    fn drop(&mut self) {
        self.replicas.remove_pin(self.peer, self.doc);
        if !self.local {
            return;
        }
        self.replicas.broadcast(&ReplicationMsg::Unpin {
            doc:        self.doc,
            withdrawal: self.withdrawal.take().map(|record| record.signed),
        });
    }
}

/// A pin as stated: the author's proof, and the record saying the document is
/// pinned.
pub struct PinClaim {
    pub proven: Proven,
    pub record: Record,
}

/// Records or restates `source`'s pin on `doc`, spawning its guard the
/// first time. `false` if the pin was refused.
pub(super) fn spawn_pin(world: &mut World, source: &Source, claim: PinClaim) -> bool {
    let PinClaim { proven, record } = claim;
    let (doc, space) = (record.payload.doc, record.payload.space);
    let by_author = source.did.as_ref() == Some(&proven.author);
    let replicas = world.resource::<Replicas>().clone();
    let msg = source.local.then(|| ReplicationMsg::Pin {
        author: proven.proof.clone(),
        record: record.signed.clone(),
    });
    if let Err(err) = replicas.add_pin(source.id, source.did.as_ref(), proven, record) {
        warn!(%doc, %err, "pin refused");
        return false;
    }
    if let Some(msg) = msg {
        replicas.broadcast(&msg);
    }
    if find_guard::<PinState>(world, source.entity, |p| p.doc == doc).is_some() {
        return true;
    }
    let anchor = doc_anchor(world, doc, space);
    world.spawn((
        PinState {
            peer: source.id,
            doc,
            local: source.local,
            by_author,
            withdrawal: None,
            replicas,
        },
        ForDoc(anchor),
        FromPeer(source.entity),
    ));
    true
}

pub(super) fn clear_pin(world: &mut World, peer_ent: Entity, doc: DocId) {
    if let Some(e) = find_guard::<PinState>(world, peer_ent, |p| p.doc == doc) {
        world.despawn(e);
    }
}

/// Pins a document the local peer has loaded, carrying the proof the
/// document's first pin did. The pin goes, and is unpinned to everyone, when
/// the document's anchor does. `false` if the pin was refused.
pub(crate) fn pin_loaded(world: &mut World, doc: DocId) -> bool {
    let Some(view) = world.get_resource::<SpaceView>().cloned() else {
        return true;
    };
    let Some(proven) = view.replicas().proven(doc) else {
        return true;
    };
    let Some(record) = view.replicas().pinned_record(doc) else {
        return true;
    };
    let me = Source {
        entity: local_peer_entity(world),
        id:     view.me(),
        did:    Some(view.did().clone()),
        local:  true,
    };
    let claim = PinClaim { proven, record };
    spawn_pin(world, &me, claim)
}

/// The local peer's pin guards on documents it does not author.
fn local_non_author_pins(world: &World) -> Vec<(Entity, DocId)> {
    let Some(local) = index::lookup::<LocalPeer>(world, ()) else {
        return Vec::new();
    };
    let Some(guards) = world.get::<PeerGuards>(local) else {
        return Vec::new();
    };
    guards
        .iter()
        .filter_map(|e| {
            let pin = world.get::<PinState>(e)?;
            (!pin.by_author).then_some((e, pin.doc))
        })
        .collect()
}

/// Drops the local peer's pin on `doc` when it is not the author's, because
/// the author withdrew it. Dropping the guard broadcasts the unpin with
/// `withdrawal`.
fn drop_local_pin(world: &mut World, doc: DocId, withdrawal: &Record) {
    for (e, pinned) in local_non_author_pins(world) {
        if pinned == doc {
            if let Some(mut pin) = world.get_mut::<PinState>(e) {
                pin.withdrawal = Some(withdrawal.clone());
            }
            world.despawn(e);
        }
    }
}

/// Keeps `record`, heard from `source`, if it is the newest for its document,
/// and drops the local non-author pin if the newest is now a withdrawal.
pub(super) fn note_record(world: &mut World, record: &Record, source: EndpointId) {
    let replicas = world.resource::<Replicas>().clone();
    replicas.observe_record(record, source);
    let doc = record.payload.doc;
    if let Some(withdrawal) = replicas.withdrawal(doc) {
        drop_local_pin(world, doc, &withdrawal);
    }
}

/// Withdraws `doc`: signs, with the document's namespace `key`, a record
/// saying it is no longer pinned, stamped past every record known, and drops
/// the local pin with it, so every peer unloads the document. Nothing is
/// withdrawn for a document no peer pins.
pub(crate) fn withdraw_pin(
    world: &mut World,
    me: EndpointId,
    doc: DocId,
    key: &NamespaceSecret,
) -> Result<(), PinError> {
    let replicas = world.resource::<Replicas>().clone();
    let Some(space) = replicas.space_of(doc) else {
        return Ok(());
    };
    let floor = replicas.newest_record_at(doc);
    let at = clock::current_micros().max(floor.map_or(0, |at| at.saturating_add(1)));
    let payload = PinRecord {
        doc,
        space,
        at,
        pinned: false,
    };
    let record = Record::sign(key, payload).map_err(|err| {
        warn!(%doc, %err, "withdrawal not signed");
        PinError::Unsigned
    })?;
    replicas.observe_record(&record, me);
    let guard = index::lookup::<LocalPeer>(world, ())
        .and_then(|local| find_guard::<PinState>(world, local, |pin| pin.doc == doc));
    match guard {
        Some(e) => {
            if let Some(mut pin) = world.get_mut::<PinState>(e) {
                pin.withdrawal = Some(record);
            }
            world.despawn(e);
        }
        None => replicas.broadcast(&ReplicationMsg::Unpin {
            doc,
            withdrawal: Some(record.signed),
        }),
    }
    Ok(())
}

impl LocalReplica {
    /// Withdraws `doc`, which this node authors, from its space, so every peer
    /// unloads it. Unloading a document or leaving its space withdraws
    /// nothing; deleting it is what calls this.
    pub async fn withdraw(&self, doc: DocId) -> Result<(), PinError> {
        let store = self
            .async_world
            .commands()
            .send_with(|world: &mut World| world.get_resource::<DataStore>().map(|s| s.0.clone()))
            .await
            .ok_or(PinError::Unavailable)?
            .ok_or(PinError::NoStore)?;
        let key = store
            .write_key(NamespaceId::from(&doc.0))
            .await
            .map_err(|err| PinError::NotAuthor(err.into()))?
            .ok_or(PinError::NotAuthor(AuthorshipError::NotWritable))?;
        let me = self.me;
        self.async_world
            .commands()
            .send_with(move |world: &mut World| withdraw_pin(world, me, doc, &key))
            .await
            .ok_or(PinError::Unavailable)?
    }
}
