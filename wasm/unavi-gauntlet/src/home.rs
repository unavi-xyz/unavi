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
            eprintln!("no root doc, cannot travel home");
            return;
        };
        self.pending = Some(wds.get(&root, "home"));
    }

    pub fn fixed_update(&mut self) {
        if let Some(fut) = &self.pending
            && let Some(result) = fut.poll()
        {
            self.pending = None;
            match result {
                Ok(Some(bytes)) => match home_namespace(&bytes) {
                    Some(ns) => {
                        if let Err(err) = travel(&ns) {
                            eprintln!("travel home failed: {err:?}");
                        }
                    }
                    None => eprintln!("malformed home space ref"),
                },
                Ok(None) => eprintln!("no home space set"),
                Err(()) => eprintln!("home read error"),
            }
        }
    }
}

/// A home entry is a version-prefixed postcard `SpaceRef` whose first field is
/// the 32-byte space namespace; extract it without the full typed struct.
fn home_namespace(bytes: &[u8]) -> Option<Vec<u8>> {
    let (_version, rest) = postcard::take_from_bytes::<u32>(bytes).ok()?;
    rest.get(..32).map(<[u8]>::to_vec)
}
