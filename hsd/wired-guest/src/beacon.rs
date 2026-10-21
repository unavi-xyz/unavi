//! The beacon-to-gate contract.
//!
//! A beacon announces its space and body on [`BEACON_CHANNEL`]. A gate
//! announces its socket and what sits in it on [`SOCKET_CHANNEL`]. Each side
//! reads the other's pose from the named prim, so neither message carries a
//! position.

use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

use serde::{
    Deserialize,
    Serialize,
};

/// A `document-id`'s four words.
pub type DocumentId = (u64, u64, u64, u64);
/// A `prim-id`'s two words.
pub type PrimId = (u64, u64);

/// Channel a beacon announces [`Announce`] on, spatially around its body.
pub const BEACON_CHANNEL: &str = "unavi:beacon/id";
/// Channel a gate announces [`Socket`] on, spatially around its seat.
pub const SOCKET_CHANNEL: &str = "unavi:gate/socket";

/// Fixed ticks between announcements on either channel.
pub const ANNOUNCE_TICKS: u32 = 8;

/// A beacon's edge, in metres.
pub const BEACON_SIZE: f32 = 0.095;
/// Metres around a socket within which a beacon thrown or dropped is drawn
/// in. A gate announces its socket over this radius.
pub const PULL_RADIUS: f32 = 1.2;

/// The custom property a minted beacon commits to say when it was planted.
///
/// Named `group/field`. Every peer reads the same stamp, so every peer agrees
/// how far into its hover a beacon is, however late it joined.
pub const PLANTED_KEY: &str = "unavi/planted";

/// The custom property on a beacon's body holding the yaw its fold-out gate
/// faces, as four little-endian bytes of radians, while its author has armed
/// it. Any other length is disarmed.
pub const ARMED_KEY: &str = "state/armed";

/// A beacon's announcement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Announce {
    /// The space it opens a gate to.
    pub space: DocumentId,
    /// The prim carrying its collider, in the sender's document.
    pub body:  PrimId,
    /// Whether the local user's grip is on it.
    pub held:  bool,
}

/// A gate's announcement of its socket.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Socket {
    /// The prim marking the seat point, in the sender's document. A seated
    /// beacon squares itself to its rotation.
    pub seat:     PrimId,
    /// The structure beside the seat that a beacon drawn in must go around.
    pub backing:  Backing,
    /// The beacon document seated in it.
    pub occupant: Option<DocumentId>,
    /// The space the gate's portal opens onto, whatever opened it.
    pub leads:    Option<DocumentId>,
}

/// A solid block on the seat's +X side, in the seat's frame: everything past
/// `from` along X and within `half_depth` along Z, from the floor up past the
/// seat.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Backing {
    pub from:       f32,
    pub half_depth: f32,
}

macro_rules! wire {
    ($ty:ty) => {
        impl $ty {
            pub fn encode(&self) -> postcard::Result<Vec<u8>> {
                postcard::to_allocvec(self)
            }

            /// `None` for bytes that are not one.
            #[must_use]
            pub fn decode(bytes: &[u8]) -> Option<Self> {
                postcard::from_bytes(bytes).ok()
            }
        }
    };
}

wire!(Announce);
wire!(Socket);

/// The wall clock in milliseconds since the Unix epoch, which is what
/// [`planted_bytes`] stamps.
#[must_use]
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_millis() as u64)
}

/// `unix_ms` as the bytes committed under [`PLANTED_KEY`].
#[must_use]
pub const fn planted_bytes(unix_ms: u64) -> [u8; 8] {
    unix_ms.to_le_bytes()
}

/// Seconds between a committed [`PLANTED_KEY`] stamp and `now_ms`. A stamp
/// from the future, which a peer with a slow clock reads, counts as just
/// planted. None when the bytes are not a stamp.
#[must_use]
pub fn age_secs(planted: &[u8], now_ms: u64) -> Option<f32> {
    let planted = u64::from_le_bytes(planted.try_into().ok()?);
    Some(now_ms.saturating_sub(planted) as f32 / 1000.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stamp_reads_back_as_its_age() {
        let stamp = planted_bytes(10_000);
        assert_eq!(age_secs(&stamp, 12_500), Some(2.5));
    }

    #[test]
    fn a_future_stamp_is_just_planted() {
        let stamp = planted_bytes(10_000);
        assert_eq!(age_secs(&stamp, 9_000), Some(0.0));
    }

    #[test]
    fn bytes_that_are_not_a_stamp_have_no_age() {
        assert_eq!(age_secs(&[1, 2, 3], 5), None);
    }
}
