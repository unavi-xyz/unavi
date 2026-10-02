//! Home: the fixed point, and the only slot that goes somewhere on its own.
//!
//! Travel is consequential, so the mote is a cast rather than a group holding
//! a confirmation — the fill ring is the confirmation.

use crate::{
    unavi::host::node_storage::{
        self,
        PendingValue,
    },
    wired::portal::portals::travel,
};

/// Reads the local player's home space ref from their root document, then
/// travels to its namespace.
#[derive(Default)]
pub struct Home {
    pending: Option<PendingValue>,
}

impl Home {
    pub fn request(&mut self) {
        let root = match node_storage::root_document() {
            Ok(Some(root)) => root,
            Ok(None) => {
                eprintln!("halo: no root document yet, cannot travel home");
                return;
            }
            Err(err) => {
                eprintln!("halo: root_document: {err:?}");
                return;
            }
        };
        match node_storage::get(root, "home") {
            Ok(pending) => self.pending = Some(pending),
            Err(err) => eprintln!("halo: get home: {err:?}"),
        }
    }

    pub fn fixed_update(&mut self) {
        let Some(pending) = &self.pending else {
            return;
        };
        let Some(result) = pending.poll() else {
            return;
        };
        self.pending = None;

        match result {
            Ok(Some(bytes)) => match namespace(&bytes) {
                Some(ns) => {
                    if let Err(err) = travel(ns) {
                        eprintln!("halo: travel home failed: {err:?}");
                    }
                }
                None => eprintln!("halo: malformed home space ref"),
            },
            Ok(None) => eprintln!("halo: no home space set"),
            Err(err) => eprintln!("halo: home read error: {err:?}"),
        }
    }
}

/// A home entry is a version-prefixed postcard `SpaceRef` whose first field is
/// the 32-byte space namespace; extract it without the full typed struct, then
/// read it as a `document-id`'s four little-endian words.
fn namespace(bytes: &[u8]) -> Option<(u64, u64, u64, u64)> {
    let (_version, rest) = postcard::take_from_bytes::<u32>(bytes).ok()?;
    let bytes: &[u8; 32] = rest.get(..32)?.try_into().ok()?;
    Some((
        u64::from_le_bytes(bytes[0..8].try_into().ok()?),
        u64::from_le_bytes(bytes[8..16].try_into().ok()?),
        u64::from_le_bytes(bytes[16..24].try_into().ok()?),
        u64::from_le_bytes(bytes[24..32].try_into().ok()?),
    ))
}
