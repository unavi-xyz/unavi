//! Pin guards, and the pin records and withdrawals they state.

use bevy::prelude::*;
use bevy_async::task;
use bevy_iroh::store::DataStore;
use hsd::id::DocId;
use iroh::EndpointId;
use iroh_docs::{
    NamespaceId,
    NamespaceSecret,
};

use crate::{
    authority::SpaceView,
    index,
    membership::SpaceId,
    replication::{
        clock,
        guards::{
            ForDoc,
            FromPeer,
            LocalPeer,
            LocalReplica,
            PeerGuards,
            Stater,
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

#[derive(Component)]
pub struct PinState {
    peer:       EndpointId,
    doc:        DocId,
    space:      SpaceId,
    local:      bool,
    /// Signs and sends the author's withdrawal when this pin is the last of
    /// the author's to go.
    signer:     Option<LocalReplica>,
    /// The withdrawal a non-author pin relays when it drops.
    withdrawal: Option<Record>,
    /// Held because a drop takes no arguments.
    replicas:   Replicas,
}

impl Drop for PinState {
    fn drop(&mut self) {
        let by_author = self.replicas.pins_as_author(self.peer, self.doc);
        let last = self.replicas.is_sole_author_pin(self.peer, self.doc);
        self.replicas.remove_pin(self.peer, self.doc);
        if !self.local {
            return;
        }
        if by_author
            && last
            && let Some(signer) = self.signer.take()
        {
            // Whether the author removed the document or left its space is
            // known only once the despawn that dropped this pin is over.
            signer.withdraw(self.doc, self.space);
            return;
        }
        let withdrawal = if by_author {
            None
        } else {
            self.withdrawal.take()
        };
        self.replicas.broadcast(&ReplicationMsg::Unpin {
            doc:        self.doc,
            withdrawal: withdrawal.map(|record| record.signed),
        });
    }
}

/// A pin as stated: the author's proof, and the record saying the document is
/// pinned.
pub struct PinClaim {
    pub proven: Proven,
    pub record: Record,
    /// Set for the local author's own pin.
    pub signer: Option<LocalReplica>,
}

pub(super) fn spawn_pin(
    world: &mut World,
    stater: &Stater,
    doc: DocId,
    space: SpaceId,
    claim: PinClaim,
) -> bool {
    if find_guard::<PinState>(world, stater.entity, |p| p.doc == doc).is_some() {
        return true;
    }
    let PinClaim {
        proven: Proven { author, proof },
        record,
        signer,
    } = claim;
    let replicas = world.resource::<Replicas>().clone();
    let msg = stater.local.then(|| ReplicationMsg::Pin {
        doc,
        space,
        author: proof.clone(),
        record: record.signed.clone(),
    });
    let at = clock::current_micros();
    if let Err(err) = replicas.add_pin(
        stater.id,
        stater.did.as_ref(),
        doc,
        space,
        author,
        proof,
        at,
    ) {
        warn!(%doc, %err, "pin refused");
        return false;
    }
    replicas.attach_record(stater.id, doc, &record);
    if let Some(msg) = msg {
        replicas.broadcast(&msg);
    }
    let anchor = doc_anchor(world, doc, space);
    world.spawn((
        PinState {
            peer: stater.id,
            doc,
            space,
            local: stater.local,
            signer,
            withdrawal: None,
            replicas,
        },
        ForDoc(anchor),
        FromPeer(stater.entity),
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
pub(crate) fn pin_loaded(world: &mut World, doc: DocId, space: SpaceId) -> bool {
    let Some(view) = world.get_resource::<SpaceView>().cloned() else {
        return true;
    };
    let Some(proven) = view.replicas().proven(doc) else {
        return true;
    };
    let Some(record) = view.replicas().pinned_record(doc) else {
        return true;
    };
    let me = Stater {
        entity: local_peer_entity(world),
        id:     view.me(),
        did:    Some(view.did().clone()),
        local:  true,
    };
    let claim = PinClaim {
        proven,
        record,
        signer: None,
    };
    spawn_pin(world, &me, doc, space, claim)
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
            (!pin.replicas.pins_as_author(pin.peer, pin.doc)).then_some((e, pin.doc))
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

/// Sends the unpin that ends `me`'s last author pin on `doc`. It carries a
/// withdrawal signed by `key`, stamped past every record known, only if the
/// local peer is still in `space`: leaving a space removes nothing.
pub(crate) fn finish_withdrawal(
    world: &World,
    me: EndpointId,
    doc: DocId,
    space: SpaceId,
    key: Option<&NamespaceSecret>,
) {
    let replicas = world.resource::<Replicas>().clone();
    let withdrawal = key.filter(|_| replicas.in_space(space)).and_then(|key| {
        let floor = replicas.newest_record_at(doc);
        let at = clock::current_micros().max(floor.map_or(0, |at| at.saturating_add(1)));
        let payload = PinRecord {
            doc,
            space,
            at,
            pinned: false,
        };
        match Record::sign(key, payload) {
            Ok(record) => Some(record),
            Err(err) => {
                warn!(%doc, %err, "withdrawal not signed; unpinning without one");
                None
            }
        }
    });
    if let Some(record) = &withdrawal {
        replicas.observe_record(record, me);
    }
    replicas.broadcast(&ReplicationMsg::Unpin {
        doc,
        withdrawal: withdrawal.map(|record| record.signed),
    });
}

impl LocalReplica {
    /// Signs and sends the withdrawal ending this node's last author pin on
    /// `doc`. The namespace key is fetched, used and dropped by the task.
    pub(super) fn withdraw(&self, doc: DocId, space: SpaceId) {
        let this = self.clone();
        task::spawn(async move {
            let key = this.namespace_key(doc).await;
            let me = this.me;
            let sent = this
                .async_world
                .commands()
                .push(move |world: &mut World| {
                    finish_withdrawal(world, me, doc, space, key.as_ref());
                })
                .send()
                .await;
            if sent.is_err() {
                warn!(%doc, "the world is gone; unpin not sent");
            }
        });
    }

    async fn namespace_key(&self, doc: DocId) -> Option<NamespaceSecret> {
        let store = self
            .async_world
            .commands()
            .send_with(|world: &mut World| world.get_resource::<DataStore>().map(|s| s.0.clone()))
            .await
            .flatten();
        let Some(store) = store else {
            warn!(%doc, "no local store to sign a withdrawal with");
            return None;
        };
        match store.write_key(NamespaceId::from(&doc.0)).await {
            Ok(Some(key)) => Some(key),
            Ok(None) => {
                warn!(%doc, "this store holds no write key to sign a withdrawal with");
                None
            }
            Err(err) => {
                warn!(%doc, %err, "write key not fetched for a withdrawal");
                None
            }
        }
    }
}
