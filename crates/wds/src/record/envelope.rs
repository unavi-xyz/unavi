use loro::{ExportMode, LoroDoc, VersionVector};
use serde::{Deserialize, Serialize};
use xdid::core::did::Did;

use crate::signed_bytes::Signable;

#[derive(Serialize, Deserialize)]
pub struct Envelope {
    author: Did,
    from: VersionVector,
    to: VersionVector,
    ops: Vec<u8>,
}

impl Signable for Envelope {}

impl Envelope {
    pub fn all_updates(author: Did, doc: &LoroDoc) -> anyhow::Result<Self> {
        let from = VersionVector::new();
        let to = doc.oplog_vv();
        let ops = doc.export(ExportMode::all_updates())?;
        Ok(Self {
            author,
            from,
            to,
            ops,
        })
    }

    pub const fn author(&self) -> &Did {
        &self.author
    }

    pub const fn start_vv(&self) -> &VersionVector {
        &self.from
    }

    pub const fn end_vv(&self) -> &VersionVector {
        &self.to
    }

    pub fn ops(&self) -> &[u8] {
        &self.ops
    }
}
