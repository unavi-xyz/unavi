//! Which of a peer's presence and leave broadcasts for a space is newest, by
//! the sender's signed issue time, so a delayed broadcast cannot undo a newer
//! one.

use std::time::Duration;

use bevy::platform::collections::HashMap;
use iroh::EndpointId;

use crate::membership::SpaceId;

/// Most (peer, space) pairs remembered. Stamps are free to sign, so this is
/// what bounds a stranger's memory cost.
const MAX_STAMPS: usize = 1024;

struct Stamp {
    issued_at: u64,
    left:      bool,
}

/// The newest broadcast applied for each (peer, space).
#[derive(Default)]
pub struct Stamps(HashMap<(EndpointId, SpaceId), Stamp>);

impl Stamps {
    /// Records the broadcast and returns whether it is newer than the last
    /// applied for the pair. A leave wins a tie with a presence, so a peer
    /// that leaves within the second of its last presence is gone.
    pub fn admit(&mut self, key: (EndpointId, SpaceId), issued_at: u64, left: bool) -> bool {
        match self.0.get_mut(&key) {
            Some(last) => {
                let older = if left {
                    issued_at < last.issued_at
                } else {
                    issued_at < last.issued_at || (issued_at == last.issued_at && last.left)
                };
                if older {
                    return false;
                }
                *last = Stamp { issued_at, left };
            }
            None => {
                // A full table forgets nothing early; the broadcast still
                // applies, it just cannot be ordered against later ones.
                if self.0.len() < MAX_STAMPS {
                    self.0.insert(key, Stamp { issued_at, left });
                }
            }
        }
        true
    }

    /// Forgets stamps older than `max_age` before `now`, which the
    /// broadcast age check would reject anyway.
    pub fn prune(&mut self, now: u64, max_age: Duration) {
        let limit = now.saturating_sub(max_age.as_secs());
        self.0.retain(|_, stamp| stamp.issued_at >= limit);
    }
}
