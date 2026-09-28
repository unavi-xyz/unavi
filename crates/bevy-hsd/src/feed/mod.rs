//! The live path from a document's store into its
//! [`HsdState`](hsd::state::HsdState). Every durable layer a state composes
//! is projected from here.

use std::collections::BTreeMap;

use async_channel::{
    Receiver,
    Sender,
    TryRecvError,
};
use bevy::prelude::*;
use hsd::{
    attributes::reference::LayerKey,
    state::entry::Entry,
};
use n0_future::FutureExt;
use unavi_util::async_task::spawn_async_task;
use wds::document::Document;

use crate::{
    Hsd,
    HsdHeld,
    HsdNamespace,
    HsdPrimIndex,
    HsdSource,
};

mod reader;

/// Deltas buffered between the reader and the frame that projects them.
const DELTA_CAPACITY: usize = 1024;

pub(crate) enum Delta {
    /// A key's winner. An empty value is a key holding none.
    Entry(Entry),
    /// The first read is complete.
    Synced,
}

/// When a feed counts its first read complete.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ready {
    /// Once the store's current entries are read.
    Snapshot,
    /// Once a sync with a peer has finished and its content has downloaded,
    /// or once the snapshot is read if the store already held a prim.
    RemoteSync,
}

/// Projects a namespace's winners into the document on this entity for as
/// long as the component lives.
#[derive(Component)]
pub struct DocFeed {
    rx:      Receiver<Delta>,
    /// Dropping it stops the reader.
    _cancel: Sender<()>,
    synced:  bool,
}

impl DocFeed {
    /// Subscribes to `doc` before anything else reads it, so a feed made
    /// ahead of a sync sees every entry that sync brings.
    #[must_use]
    pub fn spawn(doc: Document, ready: Ready) -> Self {
        let (tx, rx) = async_channel::bounded(DELTA_CAPACITY);
        let (cancel, cancelled) = async_channel::bounded::<()>(1);

        let id = doc.id();
        let read = async move {
            if let Err(err) = reader::Reader::new(doc, tx).run(ready).await {
                warn!(%id, ?err, "document feed stopped");
            }
        };
        let stop = async move {
            // Nothing is sent, so the receive ends only when the feed drops.
            match cancelled.recv().await {
                Ok(()) | Err(_) => {}
            }
        };
        spawn_async_task(read.or(stop));

        Self {
            rx,
            _cancel: cancel,
            synced: false,
        }
    }

    #[must_use]
    pub const fn is_synced(&self) -> bool {
        self.synced
    }

    /// The latest winner per key since the last drain.
    fn drain(&mut self) -> BTreeMap<String, Entry> {
        let mut out = BTreeMap::new();
        loop {
            match self.rx.try_recv() {
                Ok(Delta::Entry(entry)) => {
                    out.insert(entry.key.clone(), entry);
                }
                Ok(Delta::Synced) => self.synced = true,
                Err(TryRecvError::Empty | TryRecvError::Closed) => return out,
            }
        }
    }
}

/// Feeds a namespace-backed document the moment it gets its namespace, unless
/// the same insert brought a feed of its own.
pub(crate) fn feed_namespace(
    trigger: On<Add, HsdNamespace>,
    docs: Query<&HsdNamespace, Without<DocFeed>>,
    mut commands: Commands,
) {
    if let Ok(namespace) = docs.get(trigger.entity) {
        commands
            .entity(trigger.entity)
            .insert(DocFeed::spawn(namespace.0.clone(), Ready::Snapshot));
    }
}

/// Projects what each feed read since the last frame. An override entry is
/// also projected into every reference its site realizes.
pub(crate) fn apply_doc_deltas(
    mut feeds: Query<(&mut DocFeed, AnyOf<(&Hsd, &HsdHeld)>, Option<&HsdPrimIndex>)>,
    children: Query<&Children>,
    references: Query<&Hsd, With<HsdSource>>,
) {
    for (mut feed, (live, held), index) in &mut feeds {
        let entries = feed.drain();
        if entries.is_empty() {
            continue;
        }
        let Some(state) = live.map(|doc| &doc.0).or_else(|| held.map(|doc| &doc.0)) else {
            continue;
        };

        let Ok(mut state) = state.lock() else {
            warn!("scene state poisoned");
            continue;
        };
        for entry in entries.values() {
            if let Err(err) = state.project(entry) {
                warn!(key = %entry.key, ?err, "dropping an unreadable entry");
            }
        }
        drop(state);

        let Some(index) = index else {
            continue;
        };
        for entry in entries.values() {
            let Some((site, _)) = LayerKey::parse_key(&entry.key) else {
                continue;
            };
            let Some(site_children) = index.0.get(&site).and_then(|e| children.get(*e).ok()) else {
                continue;
            };
            for reference in references.iter_many(site_children) {
                let Ok(mut state) = reference.0.lock() else {
                    warn!("scene state poisoned");
                    continue;
                };
                if let Err(err) = state.project_override(entry) {
                    warn!(key = %entry.key, ?err, "dropping an unreadable override");
                }
            }
        }
    }
}
