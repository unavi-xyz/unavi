//! The visor's end of the lens protocol.
//!
//! A space's scripts can emit on these channels as easily as the halo can, so
//! a request is acted on only when it comes from a document the local user
//! authors.

use unavi_lens::{
    ASK,
    DocumentId,
    Lens,
    WEAR,
    Wear,
    intake::{
        Heard,
        Host,
        Intake,
    },
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

pub enum Request {
    /// Announce every lens again.
    Ask,
    Wear(Wear),
}

/// The visor's bindings to the event queue.
struct Events;

impl Host for Events {
    type Subscription = MessageSubscription;

    fn listen(&self, senders: Option<&[DocumentId]>) -> Result<MessageSubscription, String> {
        messaging::listen(&[ASK.to_owned(), WEAR.to_owned()], senders, Scope::Global)
            .map_err(|err| format!("{err:?}"))
    }

    fn drain(&self, subscription: &MessageSubscription, max: u32) -> Vec<Heard> {
        subscription
            .drain(max)
            .into_iter()
            .map(|message| Heard {
                sender:  message.sender,
                channel: message.channel,
                payload: message.payload,
            })
            .collect()
    }

    fn is_mine(&self, document: &DocumentId) -> Option<bool> {
        unavi_lens::authored(open_document(*document).ok().flatten(), |doc| {
            authority::is_author(doc).ok()
        })
    }
}

#[derive(Default)]
pub struct Channel {
    intake: Intake<MessageSubscription>,
}

impl Channel {
    /// Starts hearing requests, so none sent from here on is missed.
    pub fn start(&mut self) {
        self.intake.start(&Events);
    }

    /// The requests heard since the last call, from documents the user
    /// authors.
    pub fn poll(&mut self) -> Vec<Request> {
        let mut requests = Vec::new();
        for heard in self.intake.poll(&Events) {
            match heard.channel.as_str() {
                ASK => requests.push(Request::Ask),
                WEAR => match unavi_lens::decode_wear(&heard.payload) {
                    Ok(wear) => requests.push(Request::Wear(wear)),
                    Err(err) => eprintln!("visor: bad wear: {err}"),
                },
                _ => {}
            }
        }
        requests
    }

    /// The user's shell document the requests come from, once it is known.
    pub fn shell(&self) -> Option<DocumentId> {
        self.intake.pinned()
    }

    /// Tells everyone listening which lenses there are and which are worn.
    pub fn announce(lenses: &[Lens]) {
        let bytes = match unavi_lens::encode_lenses(lenses) {
            Ok(bytes) => bytes,
            Err(err) => {
                eprintln!("visor: encode lenses: {err}");
                return;
            }
        };
        if let Err(err) = messaging::emit(unavi_lens::LENSES, &bytes, None, Scope::Global) {
            eprintln!("visor: announce: {err:?}");
        }
    }
}
