//! Every writer's session opinions, per document, charged to the writer and
//! held through the peers that carry them.

use std::{
    collections::{
        HashMap,
        HashSet,
        hash_map::Entry,
    },
    sync::Arc,
};

use hsd::id::DocId;
use iroh::EndpointId;
use tracing::debug;
use unavi_identity::signed::Signed;
use unavi_policy::quota::{
    Quota,
    Stock,
    StockLease,
};
use xdid::core::did::Did;

use crate::{
    membership::SpaceId,
    replication::{
        frames::MAX_BATCH_BYTES,
        message::{
            Batch,
            LayerSnapshot,
            Live,
            MAX_BATCHES_PER_LAYER,
            MAX_WRITES_PER_BATCH,
            RetainedBatch,
            SessionBatch,
        },
        session::{
            SessionError,
            SessionKey,
            cell_bytes,
        },
    },
};

/// Most writers whose layers one document holds. Below the stack's own cap,
/// which also holds the document's other layers.
pub const MAX_LAYERS_PER_DOC: usize = 48;

/// Further layers past [`MAX_LAYERS_PER_DOC`] only the document's author and
/// the local peer may take, so a crowd of strangers cannot lock them out.
pub const RESERVED_LAYERS: usize = 8;

/// Bytes a write's prim id and length prefixes take in an encoded batch, which
/// its key and value bytes do not count.
const WRITE_OVERHEAD_BYTES: u64 = 20;

/// Encoded bytes a layer's retained batches may exceed twice its live bytes
/// by before it is due a checkpoint.
const CHECKPOINT_SLACK_BYTES: u64 = 16 * 1024;

/// How much a layer must grow past a failed checkpoint, in batches, before
/// another is attempted. Growing by [`CHECKPOINT_SLACK_BYTES`] does the same.
const CHECKPOINT_RETRY_BATCHES: usize = 64;

type BatchId = u64;

/// One opinion, merged last-write-wins by `at`. A `None` value blocks its key.
struct Cell {
    value:     Option<Vec<u8>>,
    /// A clear: the writer withdrew its opinion on the key. It composes no
    /// value, so the layers beneath show through, but it is held and relayed
    /// so an older write cannot fill the gap.
    tombstone: bool,
    at:        u64,
    /// The retained batch that wrote it, and the position of the write in its
    /// `writes`, or of the clear in its `clears` for a tombstone.
    batch:     BatchId,
    index:     u16,
    lease:     StockLease,
}

/// Resizes a lease to `bytes`. Growing can fail, and the lease then stays as
/// it was.
fn fit_lease(mut lease: StockLease, bytes: u64) -> StockLease {
    if lease.resize(bytes).is_err() {
        debug!(bytes, "tombstone lease kept at its old size");
    }
    lease
}

/// A signed batch kept while `refs` cells or tombstones still point at it.
struct Retained {
    signed: Signed<SessionBatch>,
    refs:   usize,
    _lease: StockLease,
}

/// One writer's opinions on one document.
struct WriterLayer {
    space:             SpaceId,
    /// The DID the layer's batches name, which decides whether it is the
    /// author's.
    did:               Did,
    /// The peers the layer is held through.
    sources:           HashSet<EndpointId>,
    cells:             HashMap<SessionKey, Cell>,
    batches:           HashMap<BatchId, Retained>,
    next_batch:        BatchId,
    /// Bytes the cells would take in a fresh batch.
    live_bytes:        u64,
    /// Encoded bytes of the retained batches.
    held_bytes:        u64,
    /// Retained bytes and batches when the last checkpoint failed.
    checkpoint_failed: Option<(u64, usize)>,
    _doc_lease:        StockLease,
}

impl WriterLayer {
    /// Lands the charged writes and clears, as cells and tombstones pointing
    /// at the batch retained for them.
    fn apply(&mut self, batch: &Batch, charged: Charged) -> Applied {
        let at = batch.stamp;
        let mut applied = Applied {
            held: true,
            ..Applied::default()
        };
        let Some(lease) = charged.batch else {
            return applied;
        };
        let id = self.next_batch;
        self.next_batch += 1;
        self.held_bytes += batch.signed.payload_len() as u64;
        self.batches.insert(
            id,
            Retained {
                signed: batch.signed.clone(),
                refs:   0,
                _lease: lease,
            },
        );
        for (index, key, value, lease) in charged.cells {
            let unchanged = self
                .cells
                .get(&key)
                .is_some_and(|cell| !cell.tombstone && cell.value == value);
            if !unchanged {
                applied.set.push((key.clone(), value.clone(), at));
            }
            self.insert_cell(
                key,
                Cell {
                    value,
                    tombstone: false,
                    at,
                    batch: id,
                    index,
                    lease,
                },
            );
        }
        for (index, key, lease) in charged.tombstones {
            let old = self.take_cell(&key);
            if old.as_ref().is_some_and(|cell| !cell.tombstone) {
                applied.cleared.push(key.clone());
            }
            // A tombstone replacing a cell takes over its lease, so a writer
            // at its quota can still clear.
            let Some(lease) = lease.or_else(|| old.map(|cell| cell.lease)) else {
                debug!("tombstone dropped: no lease");
                continue;
            };
            let lease = fit_lease(lease, cell_bytes(&key, None));
            self.insert_cell(
                key,
                Cell {
                    value: None,
                    tombstone: true,
                    at,
                    batch: id,
                    index,
                    lease,
                },
            );
        }
        applied
    }

    fn take_cell(&mut self, key: &SessionKey) -> Option<Cell> {
        let cell = self.cells.remove(key)?;
        self.live_bytes = self
            .live_bytes
            .saturating_sub(live_bytes(key, cell.value.as_deref()));
        self.unref(cell.batch);
        Some(cell)
    }

    fn remove_cell(&mut self, key: &SessionKey) -> bool {
        self.take_cell(key).is_some()
    }

    /// Takes the new batch's claim before releasing the old cell's, so a cell
    /// replaced from the same batch cannot drop the batch.
    fn insert_cell(&mut self, key: SessionKey, cell: Cell) {
        if let Some(retained) = self.batches.get_mut(&cell.batch) {
            retained.refs += 1;
        }
        self.remove_cell(&key);
        self.live_bytes += live_bytes(&key, cell.value.as_deref());
        self.cells.insert(key, cell);
    }

    /// Whether the retained batches have outgrown the live content, or are
    /// numerous enough that a snapshot of them would be refused.
    fn due_checkpoint(&self) -> bool {
        let batches = self.batches.len();
        if let Some((bytes, count)) = self.checkpoint_failed
            && self.held_bytes < bytes + CHECKPOINT_SLACK_BYTES
            && batches < count + CHECKPOINT_RETRY_BATCHES
        {
            return false;
        }
        // Batches a fresh checkpoint of the cells would take, which the count
        // trigger must exceed by a margin so a checkpoint cannot re-trigger it.
        let fresh = self
            .cells
            .len()
            .div_ceil(MAX_WRITES_PER_BATCH)
            .max(
                usize::try_from(self.live_bytes.div_ceil(MAX_BATCH_BYTES as u64))
                    .unwrap_or(usize::MAX),
            )
            .max(1);
        self.held_bytes > 2 * self.live_bytes + CHECKPOINT_SLACK_BYTES
            || (batches > MAX_BATCHES_PER_LAYER / 2 && batches > 4 * fresh)
    }

    /// Drops a cell's claim on its batch, and the batch once nothing claims it.
    fn unref(&mut self, id: BatchId) {
        let Entry::Occupied(mut entry) = self.batches.entry(id) else {
            return;
        };
        entry.get_mut().refs = entry.get().refs.saturating_sub(1);
        if entry.get().refs == 0 {
            let retained = entry.remove();
            self.held_bytes = self
                .held_bytes
                .saturating_sub(retained.signed.payload_len() as u64);
        }
    }
}

/// One write of a batch: its position, key and value.
type Write = (u16, SessionKey, Option<Vec<u8>>);

/// What a batch changes in a layer, before anything is charged.
struct Plan {
    /// Writes newer than the opinions held, the last of each key.
    writes:  Vec<Write>,
    /// Clears newer than the opinions held, by position in the batch's
    /// `clears`. A key the batch also writes is not cleared.
    clears:  Vec<(u16, SessionKey)>,
    /// Whether a write or clear is, or already was, the layer's current word
    /// on its key. A peer relaying a batch holds the layer only if one is.
    current: bool,
}

/// The leases a plan's writes and clears take, with the one for the batch
/// they retain.
struct Charged {
    cells:      Vec<(u16, SessionKey, Option<Vec<u8>>, StockLease)>,
    /// A tombstone replacing a cell has no lease of its own to take.
    tombstones: Vec<(u16, SessionKey, Option<StockLease>)>,
    batch:      Option<StockLease>,
}

impl Plan {
    fn of(
        held: Option<&WriterLayer>,
        batch: &Batch,
        live: Option<Live<'_>>,
    ) -> Result<Self, SessionError> {
        let at = batch.stamp;
        let live_writes = live.map(|live| live.writes.iter().copied().collect::<HashSet<_>>());
        let live_clears = live.map(|live| live.clears.iter().copied().collect::<HashSet<_>>());
        let mut wanted: Vec<Write> = Vec::with_capacity(batch.payload.writes.len());
        for (index, write) in batch.payload.writes.iter().enumerate() {
            let index = u16::try_from(index).map_err(|_| SessionError::TooLarge)?;
            if live_writes
                .as_ref()
                .is_some_and(|live| !live.contains(&index))
            {
                continue;
            }
            let key = SessionKey::new(write.prim, &write.name)?;
            wanted.push((index, key, write.value.clone()));
        }
        let last = wanted
            .iter()
            .enumerate()
            .map(|(position, (_, key, _))| (key.clone(), position))
            .collect::<HashMap<_, _>>();

        let mut writes = Vec::with_capacity(wanted.len());
        let mut current = false;
        for (position, (index, key, value)) in wanted.into_iter().enumerate() {
            if last.get(&key) != Some(&position) {
                continue;
            }
            let cell = held.and_then(|layer| layer.cells.get(&key));
            // A tombstone wins a tie, so the order two words arrive in never
            // decides between them.
            if cell.is_some_and(|cell| at < cell.at || (at == cell.at && cell.tombstone)) {
                continue;
            }
            if cell.is_some_and(|cell| at == cell.at && cell.value == value) {
                current = true;
                continue;
            }
            current = true;
            writes.push((index, key, value));
        }

        let mut wanted_clears: Vec<(u16, SessionKey)> = Vec::new();
        for (index, clear) in batch.payload.clears.iter().enumerate() {
            let index = u16::try_from(index).map_err(|_| SessionError::TooLarge)?;
            if live_clears
                .as_ref()
                .is_some_and(|live| !live.contains(&index))
            {
                continue;
            }
            let key = SessionKey::new(clear.prim, &clear.name)?;
            if !last.contains_key(&key) {
                wanted_clears.push((index, key));
            }
        }
        let last_clear = wanted_clears
            .iter()
            .enumerate()
            .map(|(position, (_, key))| (key.clone(), position))
            .collect::<HashMap<_, _>>();
        let mut clears = Vec::with_capacity(wanted_clears.len());
        for (position, (index, key)) in wanted_clears.into_iter().enumerate() {
            if last_clear.get(&key) != Some(&position) {
                continue;
            }
            let cell = held.and_then(|layer| layer.cells.get(&key));
            if cell.is_some_and(|cell| at < cell.at) {
                continue;
            }
            current = true;
            if cell.is_some_and(|cell| at == cell.at && cell.tombstone) {
                continue;
            }
            clears.push((index, key));
        }
        Ok(Self {
            writes,
            clears,
            current,
        })
    }

    const fn is_empty(&self) -> bool {
        self.writes.is_empty() && self.clears.is_empty()
    }

    /// Leases every write and the batch that will retain them, all or none.
    fn charge(
        &self,
        quota: &Arc<Quota>,
        batch: &Batch,
        held: Option<&WriterLayer>,
    ) -> Result<Charged, SessionError> {
        let mut cells = Vec::with_capacity(self.writes.len());
        for (index, key, value) in &self.writes {
            let lease = quota
                .lease(Stock::SessionMemory, cell_bytes(key, value.as_deref()))
                .map_err(|_| SessionError::QuotaExceeded)?;
            cells.push((*index, key.clone(), value.clone(), lease));
        }
        let mut tombstones = Vec::with_capacity(self.clears.len());
        for (index, key) in &self.clears {
            let replaces = held.is_some_and(|layer| layer.cells.contains_key(key));
            let lease = if replaces {
                None
            } else {
                Some(
                    quota
                        .lease(Stock::SessionMemory, cell_bytes(key, None))
                        .map_err(|_| SessionError::QuotaExceeded)?,
                )
            };
            tombstones.push((*index, key.clone(), lease));
        }
        let batch = if self.is_empty() {
            None
        } else {
            Some(
                quota
                    .lease(Stock::SessionMemory, batch.signed.payload_len() as u64)
                    .map_err(|_| SessionError::QuotaExceeded)?,
            )
        };
        Ok(Charged {
            cells,
            tombstones,
            batch,
        })
    }
}

fn live_bytes(key: &SessionKey, value: Option<&[u8]>) -> u64 {
    cell_bytes(key, value) + WRITE_OVERHEAD_BYTES
}

/// Who writes a batch, and what it is charged to.
pub(crate) struct Writer<'a> {
    pub(crate) peer:      EndpointId,
    pub(crate) did:       &'a Did,
    pub(crate) quota:     &'a Arc<Quota>,
    /// Whether the writer may take one of the [`RESERVED_LAYERS`].
    pub(crate) reserved:  bool,
    /// Whether the batch restates the layer's live cells, which may take it
    /// past [`MAX_BATCHES_PER_LAYER`] until the older batches drop.
    pub(crate) restating: bool,
}

/// One opinion as stored: its key, value and stamp.
pub(crate) type Opinion = (SessionKey, Option<Vec<u8>>, u64);

/// What applying a batch changed.
#[derive(Default)]
pub(crate) struct Applied {
    /// Opinions that were set.
    pub(crate) set:     Vec<Opinion>,
    /// Keys whose composed opinions a clear dropped.
    pub(crate) cleared: Vec<SessionKey>,
    /// Whether the writer's layer exists, with the source among its sources.
    pub(crate) held:    bool,
}

/// A layer a departing source was the last to hold.
pub(crate) struct Orphan {
    pub(crate) doc:    DocId,
    pub(crate) space:  SpaceId,
    pub(crate) writer: EndpointId,
    pub(crate) did:    Did,
}

#[derive(Default)]
pub(crate) struct SessionLayers(HashMap<DocId, HashMap<EndpointId, WriterLayer>>);

impl SessionLayers {
    /// The space any writer's layer places `doc` in.
    pub(crate) fn space_of(&self, doc: DocId) -> Option<SpaceId> {
        self.0.get(&doc)?.values().next().map(|layer| layer.space)
    }

    /// Writers holding a layer on `doc`, with the DID each layer names.
    pub(crate) fn layers(&self, doc: DocId) -> Vec<(EndpointId, Did)> {
        self.0.get(&doc).map_or_default(|layers| {
            layers
                .iter()
                .map(|(writer, layer)| (*writer, layer.did.clone()))
                .collect()
        })
    }

    /// Writers holding a layer on `doc`.
    pub(crate) fn writers(&self, doc: DocId) -> Vec<EndpointId> {
        self.0
            .get(&doc)
            .map_or_default(|layers| layers.keys().copied().collect())
    }

    /// `writer`'s opinions on `doc`, without its tombstones.
    pub(crate) fn opinions(&self, writer: EndpointId, doc: DocId) -> Vec<Opinion> {
        self.0
            .get(&doc)
            .and_then(|layers| layers.get(&writer))
            .map_or_default(|layer| {
                layer
                    .cells
                    .iter()
                    .filter(|(_, cell)| !cell.tombstone)
                    .map(|(key, cell)| (key.clone(), cell.value.clone(), cell.at))
                    .collect()
            })
    }

    /// The keys `writer` has cleared on `doc`, with the stamp of each.
    pub(crate) fn tombstones(&self, writer: EndpointId, doc: DocId) -> Vec<(SessionKey, u64)> {
        self.0
            .get(&doc)
            .and_then(|layers| layers.get(&writer))
            .map_or_default(|layer| {
                layer
                    .cells
                    .iter()
                    .filter(|(_, cell)| cell.tombstone)
                    .map(|(key, cell)| (key.clone(), cell.at))
                    .collect()
            })
    }

    /// Every layer, for inspection.
    pub(crate) fn all(&self) -> Vec<(DocId, EndpointId, Vec<Opinion>)> {
        let mut out = Vec::new();
        for (doc, layers) in &self.0 {
            for writer in layers.keys() {
                out.push((*doc, *writer, self.opinions(*writer, *doc)));
            }
        }
        out
    }

    /// The peers `writer`'s layer on `doc` is held through.
    pub(crate) fn sources(&self, writer: EndpointId, doc: DocId) -> Vec<EndpointId> {
        self.0
            .get(&doc)
            .and_then(|layers| layers.get(&writer))
            .map_or_default(|layer| layer.sources.iter().copied().collect())
    }

    /// Every document `writer` has a layer on, with each source of it.
    pub(crate) fn sources_of_writer(&self, writer: EndpointId) -> Vec<(DocId, EndpointId)> {
        let mut out = Vec::new();
        for (doc, layers) in &self.0 {
            if let Some(layer) = layers.get(&writer) {
                out.extend(layer.sources.iter().map(|source| (*doc, *source)));
            }
        }
        out
    }

    /// Layers `source` alone holds, other than `me`'s own.
    pub(crate) fn orphaned_by(&self, source: EndpointId, me: EndpointId) -> Vec<Orphan> {
        let mut out = Vec::new();
        for (doc, layers) in &self.0 {
            for (writer, layer) in layers {
                if *writer != me && layer.sources.len() == 1 && layer.sources.contains(&source) {
                    out.push(Orphan {
                        doc:    *doc,
                        space:  layer.space,
                        writer: *writer,
                        did:    layer.did.clone(),
                    });
                }
            }
        }
        out
    }

    /// Whether `writer`'s retained batches on `doc` are due a checkpoint.
    pub(crate) fn needs_checkpoint(&self, writer: EndpointId, doc: DocId) -> bool {
        self.0
            .get(&doc)
            .and_then(|layers| layers.get(&writer))
            .is_some_and(WriterLayer::due_checkpoint)
    }

    /// Records how a checkpoint of `writer`'s layer ended. A failed one is not
    /// retried until the layer has grown.
    pub(crate) fn record_checkpoint(&mut self, writer: EndpointId, doc: DocId, succeeded: bool) {
        if let Some(layer) = self
            .0
            .get_mut(&doc)
            .and_then(|layers| layers.get_mut(&writer))
        {
            layer.checkpoint_failed =
                (!succeeded).then_some((layer.held_bytes, layer.batches.len()));
        }
    }

    /// Every layer `source` holds, as it would relay them.
    pub(crate) fn snapshot_of(&self, source: EndpointId) -> Vec<(DocId, SpaceId, LayerSnapshot)> {
        let mut out = Vec::new();
        for (doc, layers) in &self.0 {
            for layer in layers.values().filter(|l| l.sources.contains(&source)) {
                let mut live: HashMap<BatchId, (Vec<u16>, Vec<u16>)> = HashMap::new();
                for cell in layer.cells.values() {
                    let (writes, clears) = live.entry(cell.batch).or_default();
                    if cell.tombstone {
                        clears.push(cell.index);
                    } else {
                        writes.push(cell.index);
                    }
                }
                let mut ids = layer.batches.keys().copied().collect::<Vec<_>>();
                ids.sort_unstable();
                let batches = ids
                    .into_iter()
                    .filter_map(|id| {
                        let (mut live, mut live_clears) = live.remove(&id)?;
                        live.sort_unstable();
                        live_clears.sort_unstable();
                        Some(RetainedBatch {
                            batch: layer.batches.get(&id)?.signed.clone(),
                            live,
                            live_clears,
                        })
                    })
                    .collect();
                out.push((*doc, layer.space, LayerSnapshot { batches }));
            }
        }
        out
    }

    /// Writes a batch into the writer's layer on `doc`, all or none, charging
    /// the writer, and records `source` as holding the layer. Only the writes
    /// and clears in `live` are applied, when given. A word older than the one
    /// held is skipped.
    pub(crate) fn write(
        &mut self,
        writer: &Writer<'_>,
        source: EndpointId,
        batch: &Batch,
        live: Option<Live<'_>>,
    ) -> Result<Applied, SessionError> {
        let Writer {
            peer,
            did,
            quota,
            reserved,
            restating,
        } = *writer;
        let SessionBatch { doc, space, .. } = &batch.payload;
        let (doc, space) = (*doc, *space);

        let layers = self.0.get(&doc);
        if layers.is_some_and(|layers| layers.values().any(|layer| layer.space != space)) {
            return Err(SessionError::WrongSpace);
        }
        let held = layers.and_then(|layers| layers.get(&peer));
        if held.is_some_and(|layer| layer.did != *did) {
            return Err(SessionError::WrongDid);
        }
        let plan = Plan::of(held, batch, live)?;
        // A peer relaying a layer holds it only for a word that is current.
        if source != peer && !plan.current {
            return Ok(Applied::default());
        }
        if !plan.is_empty()
            && !restating
            && held.is_some_and(|layer| layer.batches.len() >= MAX_BATCHES_PER_LAYER)
        {
            return Err(SessionError::QuotaExceeded);
        }

        let fresh = held.is_none();
        if fresh && plan.is_empty() {
            return Ok(Applied::default());
        }
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
        let charged = plan.charge(quota, batch, held)?;

        let layer = match self.0.entry(doc).or_default().entry(peer) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => {
                let Some(doc_lease) = doc_lease else {
                    return Err(SessionError::QuotaExceeded);
                };
                v.insert(WriterLayer {
                    space,
                    did: did.clone(),
                    sources: HashSet::new(),
                    cells: HashMap::new(),
                    batches: HashMap::new(),
                    next_batch: 0,
                    live_bytes: 0,
                    held_bytes: 0,
                    checkpoint_failed: None,
                    _doc_lease: doc_lease,
                })
            }
        };
        layer.sources.insert(source);
        Ok(layer.apply(batch, charged))
    }

    /// Records `source` as holding `writer`'s layer on `doc`. Answers whether
    /// the layer exists.
    pub(crate) fn add_source(
        &mut self,
        writer: EndpointId,
        doc: DocId,
        source: EndpointId,
    ) -> bool {
        self.0
            .get_mut(&doc)
            .and_then(|layers| layers.get_mut(&writer))
            .is_some_and(|layer| {
                layer.sources.insert(source);
                true
            })
    }

    /// Drops `source` from `writer`'s layer on `doc`, and the layer, releasing
    /// what it held, once no source is left. Answers whether the layer is
    /// gone.
    pub(crate) fn remove_source(
        &mut self,
        writer: EndpointId,
        doc: DocId,
        source: EndpointId,
    ) -> bool {
        let Some(layers) = self.0.get_mut(&doc) else {
            return true;
        };
        let Some(layer) = layers.get_mut(&writer) else {
            return true;
        };
        layer.sources.remove(&source);
        if !layer.sources.is_empty() {
            return false;
        }
        layers.remove(&writer);
        if layers.is_empty() {
            self.0.remove(&doc);
        }
        true
    }
}
