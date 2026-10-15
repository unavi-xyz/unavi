//! Every script's open input listeners, which the input bridge delivers to.

use std::sync::Arc;

use bevy::{
    platform::collections::HashMap,
    prelude::Resource,
};
use hsd::id::{
    DocId,
    PrimId,
};
use parking_lot::RwLock;

use crate::host::{
    input::InputEvent,
    queue::Queue,
};

/// What a listener hears.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Target {
    /// Input aimed at one prim or anything beneath it.
    Prim(DocId, PrimId),
    /// Every input of the local user's devices.
    Device,
}

/// Open listeners by target, so delivering an event costs a lookup rather
/// than a scan of every listener.
#[derive(Resource, Clone, Default)]
pub struct InputListeners(Arc<RwLock<Inner>>);

#[derive(Default)]
struct Inner {
    next:    u64,
    by_id:   HashMap<u64, Target>,
    targets: HashMap<Target, Vec<(u64, Arc<Queue<InputEvent>>)>>,
}

impl InputListeners {
    /// Opens a listener, answering the id that closes it.
    pub fn open(&self, target: Target, queue: Arc<Queue<InputEvent>>) -> u64 {
        let mut inner = self.0.write();
        let id = inner.next;
        inner.next += 1;
        inner.by_id.insert(id, target);
        inner.targets.entry(target).or_default().push((id, queue));
        id
    }

    pub fn close(&self, id: u64) {
        let mut inner = self.0.write();
        let Some(target) = inner.by_id.remove(&id) else {
            return;
        };
        if let Some(queues) = inner.targets.get_mut(&target) {
            queues.retain(|(open, _)| *open != id);
            if queues.is_empty() {
                inner.targets.remove(&target);
            }
        }
    }

    /// Whether anything listens to `target`, so a caller can skip building
    /// an event nobody hears.
    #[must_use]
    pub fn is_heard(&self, target: Target) -> bool {
        self.0.read().targets.contains_key(&target)
    }

    pub fn send(&self, target: Target, event: InputEvent) {
        if let Some(queues) = self.0.read().targets.get(&target) {
            for (_, queue) in queues {
                queue.push(event);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec3;
    use unavi_input::pointer::PointerKind;

    use super::*;
    use crate::host::input::{
        Action,
        Button,
        Ray,
    };

    fn event() -> InputEvent {
        InputEvent {
            pointer: PointerKind::Screen,
            action:  Action::Pressed(Button::Trigger),
            ray:     Ray {
                origin:    Vec3::ZERO,
                direction: Vec3::NEG_Z,
            },
            hit:     None,
        }
    }

    #[test]
    fn a_closed_listener_hears_nothing_and_leaves_nothing_behind() {
        let listeners = InputListeners::default();
        let queue = Arc::new(Queue::default());
        let id = listeners.open(Target::Device, Arc::clone(&queue));
        listeners.close(id);
        listeners.send(Target::Device, event());
        assert_eq!(queue.drain(8).len(), 0);
        assert!(!listeners.is_heard(Target::Device));
    }

    #[test]
    fn an_event_reaches_only_its_target() {
        let listeners = InputListeners::default();
        let (here, there) = (Arc::new(Queue::default()), Arc::new(Queue::default()));
        let target = Target::Prim(DocId([1; 32]), PrimId::new());
        let _ = listeners.open(target, Arc::clone(&here));
        let _ = listeners.open(Target::Device, Arc::clone(&there));
        listeners.send(target, event());
        assert_eq!(here.drain(8).len(), 1);
        assert_eq!(there.drain(8).len(), 0);
    }
}
