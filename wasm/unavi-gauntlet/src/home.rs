use wired_schemas::SCHEMA_HOME;

use crate::wired::{
    peer::api::self_did,
    portal::api::travel,
    wds::{
        api::get_wds,
        types::{
            QueryFilter,
            QueryFuture,
        },
    },
};

/// Finds the local player's home space record via WDS, then travels to it. The
/// home record's own id is the space id.
#[derive(Default)]
pub struct Home {
    pending: Option<QueryFuture>,
}

impl Home {
    pub fn request(&mut self) {
        let Ok(Some(did)) = self_did() else {
            eprintln!("self did unavailable, cannot travel home");
            return;
        };
        let Ok(wds) = get_wds() else {
            return;
        };
        self.pending = Some(wds.query(Some(&QueryFilter {
            creator: Some(did),
            schemas: Some(vec![SCHEMA_HOME.hash.as_bytes().to_vec()]),
        })));
    }

    pub fn fixed_update(&mut self) {
        if let Some(fut) = &self.pending
            && let Some(result) = fut.poll()
        {
            self.pending = None;
            match result {
                Ok(ids) => match ids.into_iter().next() {
                    Some(home) => {
                        if let Err(err) = travel(&home) {
                            eprintln!("travel home failed: {err:?}");
                        }
                    }
                    None => eprintln!("no home record found"),
                },
                Err(()) => eprintln!("home query error"),
            }
        }
    }
}
