//! Home: the fixed point, and the only slot that goes somewhere on its own.
//!
//! Travel is consequential, so the mote is a cast rather than a group holding
//! a confirmation — the fill ring is the confirmation.

use crate::wired::{
    portal::api::travel,
    wds::{
        api::get_wds,
        types::GetFuture,
    },
};

/// Reads the local player's home `SpaceRef` from their root doc, then travels
/// to its namespace.
#[derive(Default)]
pub struct Home {
    pending: Option<GetFuture>,
}

impl Home {
    pub fn request(&mut self) {
        let Ok(wds) = get_wds() else {
            return;
        };
        let Some(root) = wds.root_doc() else {
            eprintln!("halo: no root doc, cannot travel home");
            return;
        };
        self.pending = Some(wds.get(&root, "home"));
    }

    pub fn fixed_update(&mut self) {
        let Some(fut) = &self.pending else {
            return;
        };
        let Some(result) = fut.poll() else {
            return;
        };
        self.pending = None;

        match result {
            Ok(Some(bytes)) => match namespace(&bytes) {
                Some(ns) => {
                    if let Err(err) = travel(&ns) {
                        eprintln!("halo: travel home failed: {err:?}");
                    }
                }
                None => eprintln!("halo: malformed home space ref"),
            },
            Ok(None) => eprintln!("halo: no home space set"),
            Err(()) => eprintln!("halo: home read error"),
        }
    }
}

/// A home entry is a version-prefixed postcard `SpaceRef` whose first field is
/// the 32-byte space namespace; extract it without the full typed struct.
fn namespace(bytes: &[u8]) -> Option<Vec<u8>> {
    let (_version, rest) = postcard::take_from_bytes::<u32>(bytes).ok()?;
    rest.get(..32).map(<[u8]>::to_vec)
}
