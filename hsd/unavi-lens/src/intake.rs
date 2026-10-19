//! Who a script listens to for lens traffic.
//!
//! The channels are open to every script, so a script that floods them could
//! push the real traffic out of a listener's queue. A listener therefore starts
//! by hearing everyone, and once it has found a document the local user
//! authors, hears only that document, so nothing else shares its queue.

use crate::{
    DocumentId,
    verdicts::Verdicts,
};

/// Messages taken from a queue per tick: all it can hold.
const DRAIN: u32 = 64;

/// Fixed ticks between checks that the document being heard is still the
/// user's and still loaded, about a second.
const RECHECK_TICKS: u32 = 64;

/// One message, as far as the protocol cares.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heard {
    /// The sending script's document, `None` for the host.
    pub sender:  Option<DocumentId>,
    pub channel: String,
    pub payload: Vec<u8>,
}

/// What a script supplies: its own bindings to the event queue.
pub trait Host {
    type Subscription;

    /// Listens on the protocol's channels, to everyone or only `senders`.
    fn listen(&self, senders: Option<&[DocumentId]>) -> Result<Self::Subscription, String>;

    fn drain(&self, subscription: &Self::Subscription, max: u32) -> Vec<Heard>;

    /// Whether the local user authors `document`, `None` while it is not
    /// loaded or the host cannot say.
    fn is_mine(&self, document: &DocumentId) -> Option<bool>;
}

struct Pinned<S> {
    document:     DocumentId,
    subscription: S,
}

/// Messages from documents the local user authors.
pub struct Intake<S> {
    discovery:  Option<S>,
    pinned:     Option<Pinned<S>>,
    verdicts:   Verdicts<DocumentId>,
    recheck_in: u32,
}

impl<S> Default for Intake<S> {
    fn default() -> Self {
        Self {
            discovery:  None,
            pinned:     None,
            verdicts:   Verdicts::default(),
            recheck_in: 0,
        }
    }
}

impl<S> Intake<S> {
    /// Starts listening now rather than at the first poll, so nothing sent
    /// from here on is missed.
    pub fn start<H: Host<Subscription = S>>(&mut self, host: &H) {
        self.listen_to_all(host);
    }

    /// The trusted messages heard since the last call.
    pub fn poll<H: Host<Subscription = S>>(&mut self, host: &H) -> Vec<Heard> {
        self.verdicts.tick();
        self.recheck(host);
        if let Some(pinned) = &self.pinned {
            return host.drain(&pinned.subscription, DRAIN);
        }
        self.discover(host)
    }

    /// Lets go of the pinned document when it is no longer the user's or no
    /// longer loaded, so the next poll looks for one again.
    fn recheck<H: Host<Subscription = S>>(&mut self, host: &H) {
        let Some(pinned) = &self.pinned else {
            return;
        };
        if self.recheck_in > 0 {
            self.recheck_in -= 1;
            return;
        }
        self.recheck_in = RECHECK_TICKS;
        if host.is_mine(&pinned.document) != Some(true) {
            self.pinned = None;
        }
    }

    fn listen_to_all<H: Host<Subscription = S>>(&mut self, host: &H) {
        if self.discovery.is_some() || self.pinned.is_some() {
            return;
        }
        match host.listen(None) {
            Ok(subscription) => self.discovery = Some(subscription),
            Err(err) => eprintln!("lens: listen: {err}"),
        }
    }

    fn discover<H: Host<Subscription = S>>(&mut self, host: &H) -> Vec<Heard> {
        self.listen_to_all(host);
        let Some(discovery) = &self.discovery else {
            return Vec::new();
        };
        let trusted: Vec<Heard> = host
            .drain(discovery, DRAIN)
            .into_iter()
            .filter(|heard| {
                self.verdicts
                    .trusts(heard.sender, |sender| host.is_mine(sender))
            })
            .collect();

        if let Some(document) = trusted.iter().find_map(|heard| heard.sender) {
            match host.listen(Some(&[document])) {
                Ok(subscription) => {
                    self.pinned = Some(Pinned {
                        document,
                        subscription,
                    });
                    self.discovery = None;
                    self.recheck_in = RECHECK_TICKS;
                }
                Err(err) => eprintln!("lens: listen to {document:?}: {err}"),
            }
        }
        trusted
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        collections::HashMap,
    };

    use super::*;

    const MINE: DocumentId = (1, 0, 0, 0);
    const SPACE: DocumentId = (2, 0, 0, 0);

    /// Subscriptions are indices into `queues`.
    #[derive(Default)]
    struct Fake {
        queues:  RefCell<Vec<Vec<Heard>>>,
        /// The sender filter each subscription was made with.
        filters: RefCell<Vec<Option<Vec<DocumentId>>>>,
        mine:    RefCell<HashMap<DocumentId, Option<bool>>>,
    }

    impl Fake {
        fn new() -> Self {
            let fake = Self::default();
            fake.mine.borrow_mut().insert(MINE, Some(true));
            fake.mine.borrow_mut().insert(SPACE, Some(false));
            fake
        }

        /// Delivers to every subscription whose filter lets the sender in.
        fn send(&self, sender: DocumentId) {
            for (index, filter) in self.filters.borrow().iter().enumerate() {
                if filter.as_ref().is_none_or(|only| only.contains(&sender)) {
                    self.queues.borrow_mut()[index].push(Heard {
                        sender:  Some(sender),
                        channel: "c".to_owned(),
                        payload: vec![],
                    });
                }
            }
        }
    }

    impl Host for Fake {
        type Subscription = usize;

        fn listen(&self, senders: Option<&[DocumentId]>) -> Result<usize, String> {
            self.filters.borrow_mut().push(senders.map(<[_]>::to_vec));
            self.queues.borrow_mut().push(Vec::new());
            Ok(self.queues.borrow().len() - 1)
        }

        fn drain(&self, subscription: &usize, _max: u32) -> Vec<Heard> {
            std::mem::take(&mut self.queues.borrow_mut()[*subscription])
        }

        fn is_mine(&self, document: &DocumentId) -> Option<bool> {
            self.mine.borrow().get(document).copied().flatten()
        }
    }

    #[test]
    fn a_space_scripts_messages_are_dropped() {
        let host = Fake::new();
        let mut intake = Intake::default();
        intake.poll(&host);
        host.send(SPACE);
        assert!(intake.poll(&host).is_empty());
    }

    #[test]
    fn the_first_trusted_sender_is_pinned_and_discovery_is_dropped() {
        let host = Fake::new();
        let mut intake = Intake::default();
        intake.poll(&host);
        host.send(MINE);
        assert_eq!(intake.poll(&host).len(), 1);
        assert_eq!(host.filters.borrow()[1], Some(vec![MINE]));

        // Discovery is no longer drained: a flood from the space never reaches
        // a queue anyone reads, and the pinned queue holds only the user's.
        host.send(SPACE);
        host.send(MINE);
        let heard = intake.poll(&host);
        assert_eq!(heard.len(), 1);
        assert_eq!(heard[0].sender, Some(MINE));
    }

    #[test]
    fn a_pinned_document_that_unloads_sends_it_back_to_discovery() {
        let host = Fake::new();
        let mut intake = Intake::default();
        intake.poll(&host);
        host.send(MINE);
        intake.poll(&host);

        host.mine.borrow_mut().insert(MINE, None);
        for _ in 0..=RECHECK_TICKS {
            intake.poll(&host);
        }
        assert!(intake.pinned.is_none());

        host.mine.borrow_mut().insert(MINE, Some(true));
        intake.poll(&host);
        host.send(MINE);
        assert_eq!(intake.poll(&host).len(), 1);
        assert!(intake.pinned.is_some());
    }
}
