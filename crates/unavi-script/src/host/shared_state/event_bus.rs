//! The message bus every script on this peer shares.

use std::{
    collections::HashSet,
    sync::{
        Arc,
        atomic::{
            AtomicBool,
            AtomicU64,
            Ordering,
        },
    },
};

use bevy::{
    math::Vec3,
    platform::collections::HashMap,
    prelude::Resource,
};
use hsd::id::DocId;
use parking_lot::RwLock;

use crate::host::{
    queue::Queue,
    shared_state::transforms::{
        AbsoluteNodeId,
        TransformSnapshots,
    },
};

/// Every open listener of every script. One [`Resource`] rather than one per
/// script, since an emit from one script fans out to all the others.
#[derive(Resource, Clone, Default)]
pub struct EventBus(Arc<Inner>);

#[derive(Default)]
struct Inner {
    listeners: RwLock<HashMap<u64, Listener>>,
    next_id:   AtomicU64,
    observer:  RwLock<Option<EmitObserver>>,
}

/// Called for every spatial emit, for a debug overlay.
pub type EmitObserver = Box<dyn Fn(&str, Vec3, f32) + Send + Sync>;

pub struct Listener {
    pub doc:      DocId,
    pub channels: Vec<String>,
    /// `None` hears every sender.
    pub from:     Option<HashSet<DocId>>,
    pub scope:    Scope,
    pub queue:    Arc<Queue<Delivery>>,
}

#[derive(Clone, Copy)]
pub enum Scope {
    Global,
    Spatial { origin: AbsoluteNodeId, radius: f32 },
}

/// One message as a listener receives it.
#[derive(Clone)]
pub struct Delivery {
    pub channel:  Arc<str>,
    pub payload:  Arc<[u8]>,
    pub sender:   DocId,
    pub distance: Option<f32>,
    pub sent_at:  f64,
    /// Shared across the audience the sender named, so exactly one member
    /// claims it. `None` for a broadcast, which no listener can take from the
    /// others.
    pub claim:    Option<Arc<AtomicBool>>,
}

/// What a script emits.
pub struct Emit<'a> {
    pub channel: &'a str,
    pub payload: Arc<[u8]>,
    pub sender:  DocId,
    pub to:      Option<&'a HashSet<DocId>>,
    pub scope:   Scope,
    pub sent_at: f64,
}

impl EventBus {
    /// Opens a listener, answering the id that closes it.
    #[must_use]
    pub fn listen(&self, listener: Listener) -> u64 {
        let id = self.0.next_id.fetch_add(1, Ordering::Relaxed);
        self.0.listeners.write().insert(id, listener);
        id
    }

    pub fn close(&self, id: u64) {
        self.0.listeners.write().remove(&id);
    }

    /// Fans `emit` out to every listener on its channel that accepts the
    /// sender and is in range, reading positions from `transforms`.
    pub fn deliver(&self, emit: &Emit, transforms: &TransformSnapshots) {
        let origin = match emit.scope {
            Scope::Spatial { origin, radius } => {
                let Some(position) = transforms.node(&origin).map(|s| s.world.translation()) else {
                    return;
                };
                self.record_emit(emit.channel, position, radius);
                Some((position, radius))
            }
            Scope::Global => None,
        };
        let claim = emit.to.map(|_| Arc::new(AtomicBool::new(false)));
        let channel: Arc<str> = Arc::from(emit.channel);

        for listener in self.0.listeners.read().values() {
            if !listener.channels.iter().any(|c| c == emit.channel)
                || emit.to.is_some_and(|to| !to.contains(&listener.doc))
                || listener
                    .from
                    .as_ref()
                    .is_some_and(|from| !from.contains(&emit.sender))
            {
                continue;
            }
            let distance = match reach(origin, listener.scope, transforms) {
                Reach::Missed => continue,
                Reach::Global => None,
                Reach::At(distance) => Some(distance),
            };
            listener.queue.push(Delivery {
                channel: Arc::clone(&channel),
                payload: Arc::clone(&emit.payload),
                sender: emit.sender,
                distance,
                sent_at: emit.sent_at,
                claim: claim.clone(),
            });
        }
    }

    /// Every spatial listener's channels, position and radius, for a debug
    /// overlay to draw.
    #[must_use]
    pub fn spatial_listeners(&self, transforms: &TransformSnapshots) -> Vec<SpatialListener> {
        self.0
            .listeners
            .read()
            .values()
            .filter_map(|listener| match listener.scope {
                Scope::Spatial { origin, radius } => {
                    transforms.node(&origin).map(|t| SpatialListener {
                        channels: listener.channels.clone(),
                        position: t.world.translation(),
                        radius,
                    })
                }
                Scope::Global => None,
            })
            .collect()
    }

    /// Installs a debug overlay's callback for every spatial emit.
    pub fn observe(&self, cb: EmitObserver) {
        *self.0.observer.write() = Some(cb);
    }

    fn record_emit(&self, channel: &str, position: Vec3, radius: f32) {
        if let Some(observer) = self.0.observer.read().as_ref() {
            observer(channel, position, radius);
        }
    }
}

/// Whether a message reaches a listener, and how far it travelled.
enum Reach {
    Missed,
    /// Reached a global listener, which has no position to measure from.
    Global,
    At(f32),
}

fn reach(origin: Option<(Vec3, f32)>, scope: Scope, transforms: &TransformSnapshots) -> Reach {
    match (origin, scope) {
        (_, Scope::Global) => Reach::Global,
        (None, Scope::Spatial { .. }) => Reach::Missed,
        (Some((from, emit_radius)), Scope::Spatial { origin, radius }) => {
            let Some(at) = transforms.node(&origin).map(|t| t.world.translation()) else {
                return Reach::Missed;
            };
            let distance = from.distance(at);
            if distance <= emit_radius + radius {
                Reach::At(distance)
            } else {
                Reach::Missed
            }
        }
    }
}

pub struct SpatialListener {
    pub channels: Vec<String>,
    pub position: Vec3,
    pub radius:   f32,
}

/// Takes `flag`, answering whether this call was the one to.
pub fn claim(flag: &AtomicBool) -> bool {
    flag.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use hsd::id::PrimId;

    use super::*;

    fn listener(doc: DocId, from: Option<HashSet<DocId>>) -> (Listener, Arc<Queue<Delivery>>) {
        let queue = Arc::new(Queue::default());
        (
            Listener {
                doc,
                channels: vec!["a:b/c".into()],
                from,
                scope: Scope::Global,
                queue: Arc::clone(&queue),
            },
            queue,
        )
    }

    fn emit(sender: DocId, to: Option<&HashSet<DocId>>) -> Emit<'_> {
        Emit {
            channel: "a:b/c",
            payload: Arc::from([1_u8].as_slice()),
            sender,
            to,
            scope: Scope::Global,
            sent_at: 0.0,
        }
    }

    #[test]
    fn an_addressed_message_is_claimed_once_across_its_audience() {
        let bus = EventBus::default();
        let (a, a_queue) = listener(DocId([1; 32]), None);
        let (b, b_queue) = listener(DocId([2; 32]), None);
        let _ = (bus.listen(a), bus.listen(b));
        let to = HashSet::from([DocId([1; 32]), DocId([2; 32])]);

        bus.deliver(
            &emit(DocId([9; 32]), Some(&to)),
            &TransformSnapshots::default(),
        );

        let first = a_queue.drain(1).pop().and_then(|d| d.claim).expect("claim");
        let second = b_queue.drain(1).pop().and_then(|d| d.claim).expect("claim");
        assert!(claim(&first));
        assert!(
            !claim(&second),
            "a sender that names an audience asks for exactly one of it to take the message"
        );
    }

    #[test]
    fn a_broadcast_cannot_be_claimed() {
        let bus = EventBus::default();
        let (a, queue) = listener(DocId([1; 32]), None);
        let _ = bus.listen(a);
        bus.deliver(&emit(DocId([9; 32]), None), &TransformSnapshots::default());
        assert!(
            queue.drain(1).pop().expect("delivered").claim.is_none(),
            "a listener that guessed the channel must not consume a broadcast from everyone else"
        );
    }

    #[test]
    fn a_listener_hears_only_the_senders_it_named() {
        let bus = EventBus::default();
        let (a, queue) = listener(DocId([1; 32]), Some(HashSet::from([DocId([7; 32])])));
        let _ = bus.listen(a);
        bus.deliver(&emit(DocId([9; 32]), None), &TransformSnapshots::default());
        assert_eq!(queue.drain(1).len(), 0);
        bus.deliver(&emit(DocId([7; 32]), None), &TransformSnapshots::default());
        assert_eq!(queue.drain(1).len(), 1);
    }

    #[test]
    fn a_closed_listener_hears_nothing() {
        let bus = EventBus::default();
        let (a, queue) = listener(DocId([1; 32]), None);
        let id = bus.listen(a);
        bus.close(id);
        bus.deliver(&emit(DocId([9; 32]), None), &TransformSnapshots::default());
        assert_eq!(queue.drain(1).len(), 0);
    }

    #[test]
    fn a_global_message_misses_a_spatial_listener() {
        let origin = AbsoluteNodeId {
            doc:  DocId([1; 32]),
            node: PrimId::new(),
        };
        assert!(matches!(
            reach(
                None,
                Scope::Spatial {
                    origin,
                    radius: 1.0,
                },
                &TransformSnapshots::default()
            ),
            Reach::Missed
        ));
    }
}
