//! Holds heard from tools.
//!
//! Any tool may hold a lens, so unlike the halo's requests these come from
//! many documents. Each sender is checked once against the user's own
//! documents, and a refusal stands for a few seconds.

use unavi_lens::{
    DocumentId,
    HOLD,
    Hold,
    authored,
    decode_hold,
    verdicts::Verdicts,
};

use crate::wired::{
    event::messaging::{
        self,
        MessageSubscription,
        Scope,
    },
    peer::authority,
    scene::document::open_document,
};

/// Messages taken from the queue per tick: all it can hold.
const DRAIN: u32 = 64;

#[derive(Default)]
pub struct Holds {
    subscription: Option<MessageSubscription>,
    verdicts:     Verdicts<DocumentId>,
    warned:       bool,
}

impl Holds {
    /// Starts hearing holds, so none sent from here on is missed.
    pub fn start(&mut self) {
        if self.subscription.is_some() {
            return;
        }
        match messaging::listen(&[HOLD.to_owned()], None, Scope::Global) {
            Ok(subscription) => self.subscription = Some(subscription),
            Err(err) => {
                if !self.warned {
                    eprintln!("visor: listen for holds: {err:?}");
                    self.warned = true;
                }
            }
        }
    }

    /// The holds heard since the last call, with the documents that sent them,
    /// from documents the user authors.
    pub fn poll(&mut self) -> Vec<(DocumentId, Hold)> {
        self.start();
        self.verdicts.tick();
        let Some(subscription) = &self.subscription else {
            return Vec::new();
        };
        let mut holds = Vec::new();
        for message in subscription.drain(DRAIN) {
            let trusted = self.verdicts.trusts(message.sender, is_mine);
            let Some(sender) = message.sender.filter(|_| trusted) else {
                continue;
            };
            match decode_hold(&message.payload) {
                Ok(hold) => holds.push((sender, hold)),
                Err(err) => eprintln!("visor: bad hold: {err}"),
            }
        }
        holds
    }
}

fn is_mine(document: &DocumentId) -> Option<bool> {
    authored(open_document(*document).ok().flatten(), |doc| {
        authority::is_author(doc).ok()
    })
}

/// Whether `document` is still loaded.
pub fn is_loaded(document: &DocumentId) -> bool {
    open_document(*document).map_or(true, |doc| doc.is_some())
}
