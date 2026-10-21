//! The shell side: the local user's filed edits, undone and redone in order.

use std::{
    cell::RefCell,
    collections::BTreeMap,
};

use crate::{
    exports::unavi::author::api::{
        GuestHistory,
        Step,
        Way,
    },
    protocol::{
        CH_FILE,
        CH_REDO,
        CH_UNDO,
        Change,
        Filed,
        PrimId,
    },
    snapshot::Snapshot,
    wired::{
        core::{
            error::Error,
            ids::DocumentId,
        },
        event::messaging::{
            self,
            MessageSubscription,
            Scope,
        },
        peer::authority::is_author,
        scene::{
            document::{
                Document,
                Edit,
                Layer,
                open_document,
            },
            properties::{
                Property,
                PropertyKey,
            },
        },
    },
};

/// Messages drained each `poll`.
const DRAIN_MAX: u32 = 16;
/// Edits kept to undo. The oldest goes first.
const MAX_ENTRIES: usize = 128;

struct Entry {
    label:   String,
    changes: Vec<Change>,
}

#[derive(Default)]
struct Stacks {
    done:   Vec<Entry>,
    undone: Vec<Entry>,
}

pub struct History {
    /// `None` when listening failed; the history then stays empty rather
    /// than trapping the shell.
    rx:     Option<MessageSubscription>,
    stacks: RefCell<Stacks>,
}

impl GuestHistory for History {
    fn new() -> Self {
        let channels = [CH_FILE.to_owned(), CH_UNDO.to_owned(), CH_REDO.to_owned()];
        let rx = match messaging::listen(&channels, None, Scope::Global) {
            Ok(rx) => Some(rx),
            Err(err) => {
                eprintln!("history: listen: {err:?}");
                None
            }
        };
        Self {
            rx,
            stacks: RefCell::default(),
        }
    }

    fn poll(&self) -> Vec<Step> {
        let Some(rx) = &self.rx else {
            return Vec::new();
        };
        let mut steps = Vec::new();
        for message in rx.drain(DRAIN_MAX) {
            // Anything replayed here is written with the shell's authority,
            // so only the local user's own scripts are heard.
            if !message.sender.is_some_and(authored) {
                continue;
            }
            match message.channel.as_str() {
                CH_FILE => match postcard::from_bytes::<Filed>(&message.payload) {
                    Ok(filed) => self.push(filed),
                    Err(err) => eprintln!("history: malformed edit: {err}"),
                },
                CH_UNDO => steps.push(Step {
                    way:   Way::Undo,
                    label: self.undo(),
                }),
                CH_REDO => steps.push(Step {
                    way:   Way::Redo,
                    label: self.redo(),
                }),
                _ => {}
            }
        }
        steps
    }

    fn undo(&self) -> Option<String> {
        self.step(Way::Undo)
    }

    fn redo(&self) -> Option<String> {
        self.step(Way::Redo)
    }
}

impl History {
    fn push(&self, filed: Filed) {
        let mut stacks = self.stacks.borrow_mut();
        stacks.undone.clear();
        stacks.done.push(Entry {
            label:   filed.label,
            changes: filed.changes,
        });
        if stacks.done.len() > MAX_ENTRIES {
            stacks.done.remove(0);
        }
    }

    /// Replays the newest entry one way and moves it to the other stack.
    fn step(&self, way: Way) -> Option<String> {
        let mut guard = self.stacks.borrow_mut();
        let stacks = &mut *guard;
        let mut entry = match way {
            Way::Undo => stacks.done.pop()?,
            Way::Redo => stacks.undone.pop()?,
        };
        // A prim built again has a new id, which every other entry naming
        // the old one must follow.
        for (doc, old, new) in replay(&mut entry.changes, way) {
            for other in stacks.done.iter_mut().chain(stacks.undone.iter_mut()) {
                for change in &mut other.changes {
                    change.rename(doc, old, new);
                }
            }
        }
        let label = entry.label.clone();
        match way {
            Way::Undo => stacks.undone.push(entry),
            Way::Redo => stacks.done.push(entry),
        }
        Some(label)
    }
}

/// Whether the local user authors `doc`. Unloaded or unanswerable is no.
fn authored(doc: DocumentId) -> bool {
    match open_document(doc) {
        Ok(Some(doc)) => is_author(&doc).unwrap_or(false),
        Ok(None) | Err(_) => false,
    }
}

/// A prim of a document built again under a new id: the document, the old
/// id and the new.
type Rename = (DocumentId, PrimId, PrimId);

/// Replays `changes` one way, newest first when undoing, and commits each
/// document once, answering every prim built again under a new id. A
/// document that is no longer loaded is skipped and said so.
fn replay(changes: &mut [Change], way: Way) -> Vec<Rename> {
    let mut keys = BTreeMap::<DocumentId, Vec<(PrimId, PropertyKey)>>::new();
    let mut renames = Vec::new();
    let order: Vec<usize> = match way {
        Way::Undo => (0..changes.len()).rev().collect(),
        Way::Redo => (0..changes.len()).collect(),
    };
    for index in order {
        let id = changes[index].doc();
        let doc = match open_document(id) {
            Ok(Some(doc)) => doc,
            Ok(None) => {
                eprintln!("history: a document this change touched is not loaded");
                continue;
            }
            Err(err) => {
                eprintln!("history: open: {err:?}");
                continue;
            }
        };
        match replay_change(&doc, &mut changes[index], way) {
            Ok(replayed) => {
                keys.entry(id).or_default().extend(replayed.keys);
                for (old, new) in replayed.renamed {
                    for change in changes.iter_mut() {
                        change.rename(id, old, new);
                    }
                    renames.push((id, old, new));
                }
            }
            Err(err) => eprintln!("history: replay: {err:?}"),
        }
    }

    for (id, keys) in keys {
        if keys.is_empty() {
            continue;
        }
        match open_document(id) {
            Ok(Some(doc)) => {
                if let Err(err) = doc.commit(&keys) {
                    eprintln!("history: commit: {err:?}");
                }
            }
            Ok(None) => eprintln!("history: a document unloaded before its commit"),
            Err(err) => eprintln!("history: open for commit: {err:?}"),
        }
    }
    renames
}

/// What replaying one change wrote.
#[derive(Default)]
struct Replayed {
    keys:    Vec<(PrimId, PropertyKey)>,
    /// Old and new ids of prims built again.
    renamed: Vec<(PrimId, PrimId)>,
}

/// Writes one change's live value.
fn replay_change(doc: &Document, change: &mut Change, way: Way) -> Result<Replayed, Error> {
    match change {
        Change::Set {
            prim,
            key,
            before,
            after,
            ..
        } => {
            let (expected, target) = match way {
                Way::Undo => (&*after, &*before),
                Way::Redo => (&*before, &*after),
            };
            if doc.get(*prim, key).as_ref() != expected.as_deref() {
                eprintln!("history: {key:?} changed since, so it is left as it is");
                return Ok(Replayed::default());
            }
            write(doc, *prim, key, target.as_deref())?;
            Ok(Replayed {
                keys:    vec![(*prim, key.clone())],
                renamed: Vec::new(),
            })
        }
        Change::Create { prims, .. } => build_or_remove(doc, prims, way == Way::Redo),
        Change::Remove { prims, .. } => build_or_remove(doc, prims, way == Way::Undo),
    }
}

/// Builds `prims` again when `build`, else removes every one of them, so no
/// descendant is left behind in the document.
fn build_or_remove(doc: &Document, prims: &mut Snapshot, build: bool) -> Result<Replayed, Error> {
    if build {
        let renamed = prims.rebuild(doc)?;
        return Ok(Replayed {
            keys: prims.keys(),
            renamed,
        });
    }
    let present = prims
        .prims()
        .rev()
        .filter(|prim| doc.contains(*prim))
        .map(Edit::Remove)
        .collect::<Vec<_>>();
    if present.is_empty() {
        eprintln!("history: the prims this change made are already gone");
        return Ok(Replayed::default());
    }
    doc.apply(Layer::Local, &present)?;
    Ok(Replayed {
        keys:    prims.parent_keys(),
        renamed: Vec::new(),
    })
}

/// States `value` for `key` live, or that it has none. No value needs the
/// shared layer: only there does clearing a key hide what is beneath it.
fn write(
    doc: &Document,
    prim: PrimId,
    key: &PropertyKey,
    value: Option<&Property>,
) -> Result<(), Error> {
    value.map_or_else(
        || doc.apply(Layer::Shared, &[Edit::Clear((prim, key.clone()))]),
        |value| doc.apply(Layer::Local, &[Edit::Set((prim, value.clone()))]),
    )
}
