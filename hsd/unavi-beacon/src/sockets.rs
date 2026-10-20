//! The gate sockets a beacon has heard, and where each one is now.

use wired_guest::{
    beacon::{
        BEACON_SIZE,
        Backing,
        DocumentId,
        PULL_RADIUS,
        PrimId,
        SOCKET_CHANNEL,
        Socket,
    },
    math::{
        Transform,
        Vec3,
    },
};

use crate::wired::{
    core::{
        error::Error,
        ids::PrimRef,
    },
    event::messaging::{
        self,
        MessageSubscription,
        Scope,
        Spatial,
    },
    scene::document::{
        Document,
        open_document,
    },
};

/// Metres past which a heard socket is forgotten until it announces again.
const FORGET_RADIUS: f32 = PULL_RADIUS + 0.5;

struct Heard {
    doc:    Document,
    socket: Socket,
    /// Its seat as last read, kept through a frame that has not drawn it.
    last:   Option<Transform>,
}

/// A socket, measured this tick.
#[derive(Clone, Copy, Debug)]
pub struct Nearby {
    /// The gate document and its seat prim.
    pub key:      (DocumentId, PrimId),
    /// Its seat, in the beacon's frame.
    pub seat:     Transform,
    pub backing:  Backing,
    pub distance: f32,
    pub occupant: Option<DocumentId>,
}

impl Nearby {
    /// Whether it is empty, or holds `beacon`.
    pub fn takes(&self, beacon: DocumentId) -> bool {
        self.occupant.is_none_or(|o| o == beacon)
    }
}

pub struct Sockets {
    rx:    MessageSubscription,
    heard: Vec<Heard>,
}

impl Sockets {
    /// Hears sockets whose pull reaches `body`.
    pub fn listen(body: PrimRef) -> anyhow::Result<Self> {
        let rx = messaging::listen(
            &[SOCKET_CHANNEL.to_owned()],
            None,
            Scope::Spatial(Spatial {
                origin: body,
                radius: BEACON_SIZE,
            }),
        )?;
        Ok(Self {
            rx,
            heard: Vec::new(),
        })
    }

    /// Takes in new announcements, then measures every heard socket from
    /// `at`, nearest first.
    pub fn measure(&mut self, at: Vec3) -> anyhow::Result<Vec<Nearby>> {
        for message in self.rx.drain(32) {
            let (Some(sender), Some(socket)) = (message.sender, Socket::decode(&message.payload))
            else {
                continue;
            };
            if let Some(heard) = self
                .heard
                .iter_mut()
                .find(|h| h.doc.id() == sender && h.socket.seat == socket.seat)
            {
                heard.socket = socket;
                continue;
            }
            if let Some(doc) = open_document(sender)? {
                self.heard.push(Heard {
                    doc,
                    socket,
                    last: None,
                });
            }
        }

        let mut nearby = Vec::with_capacity(self.heard.len());
        self.heard.retain_mut(|heard| {
            let seat = match heard.doc.world_transform(heard.socket.seat) {
                Ok(seat) => seat,
                Err(Error::NotReady) => match heard.last {
                    Some(last) => last,
                    None => return true,
                },
                Err(_) => return false,
            };
            heard.last = Some(seat);
            let distance = seat.translation.distance(at);
            if distance > FORGET_RADIUS {
                return false;
            }
            nearby.push(Nearby {
                key: (heard.doc.id(), heard.socket.seat),
                seat,
                backing: heard.socket.backing,
                distance,
                occupant: heard.socket.occupant,
            });
            true
        });
        nearby.sort_by(|a, b| a.distance.total_cmp(&b.distance));
        Ok(nearby)
    }
}
