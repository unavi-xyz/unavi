use std::{
    collections::VecDeque,
    sync::Arc,
};

use bevy::log::warn_once;
use parking_lot::Mutex;

use crate::runtime::shared::{
    Api,
    wired::input::types::InputEvent,
};

/// Events a listener may fall behind by. A script drains its listener every
/// tick, so reaching this means it has stopped reading at all.
const QUEUE_DEPTH: usize = 64;

/// A listener's backlog, written by the bridge and drained by the script.
///
/// Overflow drops the *oldest* event: a listener this far behind has already
/// lost the thread, and what the user just did matters more than what they
/// did a second ago.
#[derive(Clone, Default)]
pub struct InputQueue(Arc<Mutex<VecDeque<InputEvent>>>);

impl InputQueue {
    pub fn push(&self, event: InputEvent) {
        let mut queue = self.0.lock();
        if queue.len() >= QUEUE_DEPTH {
            queue.pop_front();
            warn_once!("an input listener is not being polled; events are being dropped");
        }
        queue.push_back(event);
    }

    #[must_use]
    pub fn pop(&self) -> Option<InputEvent> {
        self.0.lock().pop_front()
    }
}

pub struct InputListenerRes {
    pub queue: InputQueue,
}

pub async fn poll(backend: &Api, listener: u32) -> anyhow::Result<Option<InputEvent>> {
    backend
        .wired_input
        .lock()
        .await
        .listeners
        .get(listener)
        .map(|res| res.queue.pop())
        .ok_or_else(|| anyhow::anyhow!("listener not found"))
}

pub async fn drop(backend: &Api, listener: u32) -> anyhow::Result<()> {
    backend.wired_input.lock().await.listeners.remove(listener);
    Ok(())
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use unavi_input::pointer::PointerKind;

    use super::*;
    use crate::runtime::shared::wired::input::types::{
        InputAction,
        Ray,
    };

    fn event(action: InputAction) -> InputEvent {
        InputEvent {
            pointer: PointerKind::Screen,
            action,
            ray: Ray {
                origin: Vec3::ZERO,
                dir:    Vec3::NEG_Z,
            },
            hit: None,
        }
    }

    #[test]
    fn events_come_back_in_the_order_they_happened() {
        let queue = InputQueue::default();
        queue.push(event(InputAction::Press));
        queue.push(event(InputAction::Release));

        assert_eq!(queue.pop().expect("press").action, InputAction::Press);
        assert_eq!(queue.pop().expect("release").action, InputAction::Release);
        assert!(queue.pop().is_none());
    }

    #[test]
    fn a_full_queue_keeps_the_newest_events() {
        let queue = InputQueue::default();
        for _ in 0..QUEUE_DEPTH {
            queue.push(event(InputAction::Press));
        }
        queue.push(event(InputAction::Release));

        let held = std::iter::from_fn(|| queue.pop()).collect::<Vec<_>>();
        assert_eq!(held.len(), QUEUE_DEPTH, "the queue stays bounded");
        assert_eq!(
            held.last().expect("last").action,
            InputAction::Release,
            "the event that overflowed it is the one kept"
        );
    }
}
