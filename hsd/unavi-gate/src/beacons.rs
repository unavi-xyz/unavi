//! The beacons a gate has heard, and where each one is now.

use wired_guest::{
    beacon::{
        Announce,
        BEACON_CHANNEL,
        DocumentId,
        PULL_RADIUS,
    },
    math::Vec3,
};

use crate::{
    seat::Candidate,
    wired::{
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
    },
};

/// Metres from the seat point past which a heard beacon is forgotten until it
/// announces again.
const FORGET_RADIUS: f32 = PULL_RADIUS + 0.5;

struct Heard {
    doc:      Document,
    announce: Announce,
    /// Its body as last read, kept through a frame that has not drawn it.
    last:     Option<Vec3>,
}

pub struct Beacons {
    rx:    MessageSubscription,
    heard: Vec<Heard>,
}

impl Beacons {
    /// Hears beacons announcing within [`PULL_RADIUS`] of `seat`.
    pub fn listen(seat: PrimRef) -> anyhow::Result<Self> {
        let rx = messaging::listen(
            &[BEACON_CHANNEL.to_owned()],
            None,
            Scope::Spatial(Spatial {
                origin: seat,
                radius: PULL_RADIUS,
            }),
        )?;
        Ok(Self {
            rx,
            heard: Vec::new(),
        })
    }

    /// Takes in new announcements, then measures every heard beacon against
    /// `seat`, in the gate's frame. A beacon whose document is gone, or that
    /// has moved past [`FORGET_RADIUS`], is forgotten.
    pub fn measure(&mut self, seat: Vec3) -> anyhow::Result<Vec<Candidate>> {
        for message in self.rx.drain(32) {
            let (Some(sender), Some(announce)) =
                (message.sender, Announce::decode(&message.payload))
            else {
                continue;
            };
            if let Some(heard) = self.heard.iter_mut().find(|h| h.doc.id() == sender) {
                heard.announce = announce;
                continue;
            }
            if let Some(doc) = open_document(sender)? {
                self.heard.push(Heard {
                    doc,
                    announce,
                    last: None,
                });
            }
        }

        let mut near = Vec::with_capacity(self.heard.len());
        self.heard.retain_mut(|heard| {
            let body = match heard.doc.world_transform(heard.announce.body) {
                Ok(body) => body.translation,
                Err(Error::NotReady) => match heard.last {
                    Some(last) => last,
                    None => return true,
                },
                Err(_) => return false,
            };
            heard.last = Some(body);
            let distance = body.distance(seat);
            if distance > FORGET_RADIUS {
                return false;
            }
            near.push(Candidate {
                beacon: heard.doc.id(),
                space: heard.announce.space,
                distance,
            });
            true
        });
        Ok(near)
    }
}

/// The nearest of `near` that is not `except`.
pub fn nearest(near: &[Candidate], except: Option<DocumentId>) -> Option<Candidate> {
    near.iter()
        .filter(|c| Some(c.beacon) != except)
        .min_by(|a, b| a.distance.total_cmp(&b.distance))
        .copied()
}
