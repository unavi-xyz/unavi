//! Every present peer's session opinions, per document, charged to the peer.

use std::{
    collections::{
        HashMap,
        hash_map::Entry,
    },
    sync::Arc,
};

use hsd::id::DocId;
use iroh::EndpointId;
use unavi_policy::quota::{
    Quota,
    Stock,
    StockLease,
};
use xdid::core::did::Did;

use crate::{
    membership::SpaceId,
    replication::session::{
        SessionError,
        SessionKey,
        cell_bytes,
    },
};

/// Most peers whose layers one document holds. Below the stack's own cap,
/// which also holds the document's other layers.
pub const MAX_LAYERS_PER_DOC: usize = 48;

/// Further layers past [`MAX_LAYERS_PER_DOC`] only the document's author and
/// the local peer may take, so a crowd of strangers cannot lock them out.
pub const RESERVED_LAYERS: usize = 8;

/// One opinion, merged last-write-wins by `at`. A `None` value blocks its key.
struct Cell {
    value:  Option<Vec<u8>>,
    at:     u64,
    _lease: StockLease,
}

/// One peer's opinions on one document.
struct PeerLayer {
    space:      SpaceId,
    /// The DID the peer proved, which decides whether it is the author.
    did:        Option<Did>,
    cells:      HashMap<SessionKey, Cell>,
    _doc_lease: StockLease,
}

/// Who writes a batch, and what it is charged to.
pub(crate) struct Writer<'a> {
    pub(crate) peer:     EndpointId,
    pub(crate) did:      Option<&'a Did>,
    pub(crate) quota:    &'a Arc<Quota>,
    /// Whether the writer may take one of the [`RESERVED_LAYERS`].
    pub(crate) reserved: bool,
}

/// One opinion as stored: its key, value and stamp.
pub(crate) type Opinion = (SessionKey, Option<Vec<u8>>, u64);

#[derive(Default)]
pub(crate) struct SessionLayers(HashMap<DocId, HashMap<EndpointId, PeerLayer>>);

impl SessionLayers {
    /// The space any peer's layer places `doc` in.
    pub(crate) fn space_of(&self, doc: DocId) -> Option<SpaceId> {
        self.0.get(&doc)?.values().next().map(|layer| layer.space)
    }

    /// Peers holding a layer on `doc`, with the DID each proved.
    pub(crate) fn layers(&self, doc: DocId) -> Vec<(EndpointId, Option<Did>)> {
        self.0.get(&doc).map_or_default(|layers| {
            layers
                .iter()
                .map(|(peer, layer)| (*peer, layer.did.clone()))
                .collect()
        })
    }

    /// Peers holding a layer on `doc`.
    pub(crate) fn peers(&self, doc: DocId) -> Vec<EndpointId> {
        self.0
            .get(&doc)
            .map_or_default(|layers| layers.keys().copied().collect())
    }

    /// `peer`'s opinions on `doc`.
    pub(crate) fn opinions(&self, peer: EndpointId, doc: DocId) -> Vec<Opinion> {
        self.0
            .get(&doc)
            .and_then(|layers| layers.get(&peer))
            .map_or_default(|layer| {
                layer
                    .cells
                    .iter()
                    .map(|(key, cell)| (key.clone(), cell.value.clone(), cell.at))
                    .collect()
            })
    }

    /// Every document `peer` holds a layer on, with its space and opinions.
    pub(crate) fn of_peer(&self, peer: EndpointId) -> Vec<(DocId, SpaceId, Vec<Opinion>)> {
        let mut out = Vec::new();
        for (doc, layers) in &self.0 {
            if let Some(layer) = layers.get(&peer) {
                out.push((*doc, layer.space, self.opinions(peer, *doc)));
            }
        }
        out
    }

    /// Every layer, for inspection.
    pub(crate) fn all(&self) -> Vec<(DocId, EndpointId, Vec<Opinion>)> {
        let mut out = Vec::new();
        for (doc, layers) in &self.0 {
            for peer in layers.keys() {
                out.push((*doc, *peer, self.opinions(*peer, *doc)));
            }
        }
        out
    }

    /// Writes a batch into the writer's layer on `doc`, all or none, charging
    /// the writer. An opinion older than the one held is skipped. Answers the
    /// opinions that changed.
    pub(crate) fn write(
        &mut self,
        writer: &Writer<'_>,
        doc: DocId,
        space: SpaceId,
        batch: Vec<(SessionKey, Option<Vec<u8>>)>,
        at: u64,
    ) -> Result<Vec<Opinion>, SessionError> {
        let Writer {
            peer,
            did,
            quota,
            reserved,
        } = *writer;
        let layers = self.0.get(&doc);
        if layers.is_some_and(|layers| layers.values().any(|layer| layer.space != space)) {
            return Err(SessionError::WrongSpace);
        }
        let held = layers.and_then(|layers| layers.get(&peer));
        let fresh = held.is_none();
        let cap = MAX_LAYERS_PER_DOC + if reserved { RESERVED_LAYERS } else { 0 };
        if fresh && layers.is_some_and(|layers| layers.len() >= cap) {
            return Err(SessionError::QuotaExceeded);
        }
        let doc_lease = if fresh {
            Some(
                quota
                    .lease(Stock::Documents, 1)
                    .map_err(|_| SessionError::QuotaExceeded)?,
            )
        } else {
            None
        };

        let mut leased = Vec::with_capacity(batch.len());
        for (key, value) in batch {
            if held
                .and_then(|layer| layer.cells.get(&key))
                .is_some_and(|cell| at < cell.at)
            {
                continue;
            }
            let lease = quota
                .lease(Stock::SessionMemory, cell_bytes(&key, value.as_deref()))
                .map_err(|_| SessionError::QuotaExceeded)?;
            leased.push((key, value, lease));
        }

        let layer = match self.0.entry(doc).or_default().entry(peer) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => {
                let Some(doc_lease) = doc_lease else {
                    return Err(SessionError::QuotaExceeded);
                };
                v.insert(PeerLayer {
                    space,
                    did: did.cloned(),
                    cells: HashMap::new(),
                    _doc_lease: doc_lease,
                })
            }
        };
        let mut changed = Vec::with_capacity(leased.len());
        for (key, value, lease) in leased {
            changed.push((key.clone(), value.clone(), at));
            layer.cells.insert(
                key,
                Cell {
                    value,
                    at,
                    _lease: lease,
                },
            );
        }
        Ok(changed)
    }

    /// Drops `peer`'s opinions on `keys` of `doc`, releasing what they held.
    pub(crate) fn clear(&mut self, peer: EndpointId, doc: DocId, keys: &[SessionKey]) {
        if let Some(layer) = self
            .0
            .get_mut(&doc)
            .and_then(|layers| layers.get_mut(&peer))
        {
            for key in keys {
                layer.cells.remove(key);
            }
        }
    }

    /// Drops `peer`'s layer on `doc`, releasing what it held.
    pub(crate) fn remove(&mut self, peer: EndpointId, doc: DocId) {
        if let Some(layers) = self.0.get_mut(&doc) {
            layers.remove(&peer);
            if layers.is_empty() {
                self.0.remove(&doc);
            }
        }
    }
}
