use iroh::EndpointId;
use unavi_policy::quota::StockHold;

/// A KV cell, merged last-write-wins by `(at, peer)`. `value: None` is a
/// retained tombstone, so a delete keeps winning over an older live write.
///
/// Cells live on the document, never under the peer that wrote one, so a cell
/// survives exactly as long as the document does. Ownership of a document
/// migrates to the next-oldest pinner when its owner leaves; keeping the cells
/// under that peer's replica would drop the state while the content it belongs
/// to carried on.
pub(crate) struct Cell {
    pub(crate) at:    u64,
    pub(crate) peer:  EndpointId,
    pub(crate) value: Option<Vec<u8>>,
    pub(crate) hold:  StockHold,
    /// Exactly one prior version, making "revert everything peer X wrote" a
    /// scan of the cell map rather than a general undo log.
    pub(crate) prev:  Option<Box<Self>>,
}

pub(crate) fn cell_bytes(key: &str, value: Option<&[u8]>) -> u64 {
    (key.len() + value.map_or(0, <[u8]>::len)) as u64
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum KvError {
    #[error("kv key exceeds maximum length")]
    KeyTooLong,
    #[error("kv write to a peer-owned document by a non-owner")]
    NotOwner,
    #[error("kv write exceeds quota")]
    QuotaExceeded,
    #[error("kv write failed")]
    Other,
}
