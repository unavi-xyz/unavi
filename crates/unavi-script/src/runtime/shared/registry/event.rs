use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{
            AtomicBool,
            AtomicU32,
            Ordering,
        },
    },
};

use async_channel::{
    Receiver,
    Sender,
};
use bevy::{
    math::Vec3,
    prelude::Resource,
};
use hsd::id::DocId;
use parking_lot::RwLock;
use web_time::{
    SystemTime,
    UNIX_EPOCH,
};

use crate::runtime::shared::registry::transform::{
    AbsoluteNodeId,
    TransformSnapshots,
};

const RECEPTOR_CAPACITY: usize = 16;

pub const HOST_SENDER_DOC: [u8; 32] = [0u8; 32];

/// The cross-script event bus: every receptor any script has opened, and the
/// counter minting each one's id.
///
/// A [`Resource`] cloned into every script's
/// [`Api`](crate::runtime::shared::Api) rather than scoped per script — an
/// `emit` from one script must fan out to every other script's receptors,
/// which is the entire point of the bus.
#[derive(Resource, Clone, Default)]
pub struct EventBus(Arc<Inner>);

#[derive(Default)]
struct Inner {
    receptors: RwLock<HashMap<u32, ReceptorEntry>>,
    next_id:   AtomicU32,
    observer:  RwLock<Option<EmitObserver>>,
}

pub type EmitObserver = Box<dyn Fn(&str, Vec3, f32) + Send + Sync>;

impl EventBus {
    /// Registers a new receptor for `doc_id`, minting its id from the shared
    /// counter, and returns the id together with the receiving half of its
    /// channel.
    pub fn listen(
        &self,
        doc_id: DocId,
        channels: Vec<String>,
        scope: ReceptorScope,
        source_documents: Option<Vec<Vec<u8>>>,
    ) -> (u32, Receiver<InboundEvent>) {
        let (tx, rx) = async_channel::bounded(RECEPTOR_CAPACITY);
        let id = self.0.next_id.fetch_add(1, Ordering::Relaxed);
        self.0.receptors.write().insert(
            id,
            ReceptorEntry {
                channels,
                doc_id,
                scope,
                source_documents,
                tx,
            },
        );
        (id, rx)
    }

    /// Removes a receptor, but only when `doc_id` is the document that opened
    /// it.
    ///
    /// Receptor ids are otherwise unique only because of the shared counter,
    /// with no ownership check: without this, one script could drop another
    /// script's receptor by guessing or being handed a stale handle.
    pub fn drop_receptor(&self, rep: u32, doc_id: DocId) {
        let mut receptors = self.0.receptors.write();
        if receptors.get(&rep).is_some_and(|e| e.doc_id == doc_id) {
            receptors.remove(&rep);
        }
    }

    #[must_use]
    pub fn doc_has_receptor(&self, doc: DocId, channel: &str) -> bool {
        self.0
            .receptors
            .read()
            .values()
            .any(|e| e.doc_id == doc && e.channels.iter().any(|c| c == channel))
    }

    /// Delivers a host-originated event directly to every receptor `target_doc`
    /// opened on `channel`, bypassing the write and scope checks a
    /// script-originated emit is subject to.
    pub fn emit_from_host(&self, target_doc: DocId, channel: &str, payload: Vec<u8>) {
        let payload = Arc::new(payload);
        let claimed = Arc::new(AtomicBool::new(false));
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or_default();

        let receptors = self.0.receptors.read();
        for entry in receptors.values() {
            if entry.doc_id != target_doc {
                continue;
            }
            if !entry.channels.iter().any(|c| c == channel) {
                continue;
            }
            let _ = entry.tx.try_send(InboundEvent {
                channel: channel.into(),
                payload: Arc::clone(&payload),
                sender_document: HOST_SENDER_DOC.to_vec(),
                sender_scope: SenderScope::Global,
                time,
                claimed: Arc::clone(&claimed),
            });
        }
    }

    /// Fans a script-originated emit out to every receptor addressed by
    /// `channel` and the audience/source-document filters, using `resolve` to
    /// settle the parts that only the caller's
    /// [`Api`](crate::runtime::shared::Api) can answer: whether the emitter
    /// may write to a given receptor's document, and what scope the
    /// receptor sees the emitter at.
    pub fn deliver(
        &self,
        channel: &str,
        payload: &Arc<Vec<u8>>,
        sender_document: &[u8],
        time: u64,
        documents: Option<&[Vec<u8>]>,
        audience_claim: Option<&Arc<AtomicBool>>,
        mut resolve: impl FnMut(DocId, &ReceptorScope) -> Option<SenderScope>,
    ) {
        let receptors = self.0.receptors.read();
        for entry in receptors.values() {
            if !entry.channels.iter().any(|c| c == channel) {
                continue;
            }

            if let Some(docs) = documents
                && !docs
                    .iter()
                    .any(|d| d.as_slice() == entry.doc_id.0.as_slice())
            {
                continue;
            }

            if let Some(docs) = &entry.source_documents
                && !docs.iter().any(|d| d.as_slice() == sender_document)
            {
                continue;
            }

            let Some(sender_scope) = resolve(entry.doc_id, &entry.scope) else {
                continue;
            };

            let _ = entry.tx.try_send(InboundEvent {
                channel: channel.to_string(),
                payload: Arc::clone(payload),
                sender_document: sender_document.to_vec(),
                sender_scope,
                time,
                claimed: delivery_claim(audience_claim),
            });
        }
    }

    /// Every spatial receptor's channels, position and radius, for the debug
    /// overlay to draw.
    #[must_use]
    pub fn spatial_receptors(&self, transforms: &TransformSnapshots) -> Vec<SpatialReceptor> {
        self.0
            .receptors
            .read()
            .values()
            .filter_map(|entry| match &entry.scope {
                ReceptorScope::Spatial { node, radius } => {
                    transforms.node(node).map(|t| SpatialReceptor {
                        channels: entry.channels.clone(),
                        position: t.global.translation(),
                        radius:   *radius,
                    })
                }
                ReceptorScope::Global => None,
            })
            .collect()
    }

    /// Installs the debug overlay's callback for every spatial emit.
    pub fn observe(&self, cb: EmitObserver) {
        *self.0.observer.write() = Some(cb);
    }

    pub(crate) fn record_emit(&self, channel: &str, position: Vec3, radius: f32) {
        if let Some(observer) = self.0.observer.read().as_ref() {
            observer(channel, position, radius);
        }
    }
}

pub struct ReceptorEntry {
    pub channels:         Vec<String>,
    pub doc_id:           DocId,
    pub scope:            ReceptorScope,
    pub source_documents: Option<Vec<Vec<u8>>>,
    pub tx:               Sender<InboundEvent>,
}

pub enum ReceptorScope {
    Global,
    Spatial { node: AbsoluteNodeId, radius: f32 },
}

#[derive(Clone)]
pub struct InboundEvent {
    pub channel:         String,
    pub payload:         Arc<Vec<u8>>,
    pub sender_document: Vec<u8>,
    pub sender_scope:    SenderScope,
    pub time:            u64,
    pub claimed:         Arc<AtomicBool>,
}

#[derive(Clone)]
pub enum SenderScope {
    Global,
    Spatial {
        distance: f32,
        node:     AbsoluteNodeId,
    },
}

pub struct SpatialReceptor {
    pub channels: Vec<String>,
    pub position: Vec3,
    pub radius:   f32,
}

/// One claim shared across the audience an emitter named, or a claim of its own
/// per recipient when it named none.
///
/// Exclusion is only meaningful within an addressed set. Sharing one claim
/// across every receptor that guessed the channel string would let any listener
/// consume a broadcast out from under all the others.
fn delivery_claim(audience: Option<&Arc<AtomicBool>>) -> Arc<AtomicBool> {
    audience.map_or_else(|| Arc::new(AtomicBool::new(false)), Arc::clone)
}

pub(crate) fn claim(flag: &AtomicBool) -> bool {
    flag.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_addressed_emit_is_claimed_once_across_its_audience() {
        let audience = Arc::new(AtomicBool::new(false));
        let first = delivery_claim(Some(&audience));
        let second = delivery_claim(Some(&audience));

        assert!(claim(&first));
        assert!(
            !claim(&second),
            "an emitter that named an audience asked for exactly one of them to take it"
        );
    }

    #[test]
    fn a_broadcast_recipient_cannot_deny_the_others() {
        let first = delivery_claim(None);
        let second = delivery_claim(None);

        assert!(claim(&first));
        assert!(
            claim(&second),
            "a listener that guessed the channel string must not consume a broadcast \
             out from under everyone it was not addressed to"
        );
    }
}
