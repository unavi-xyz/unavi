//! Every writer's session layer, per document: its cells, the newest signed
//! state of them, and the peers it is held through, charged to the writer.

use std::{
    collections::{
        HashMap,
        HashSet,
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
    replication::{
        clock,
        message::session::{
            LayerSnapshot,
            LayerState,
            ReceivedLayer,
            ReceivedTick,
            SessionWrite,
            SignedLayer,
        },
        session::{
            SessionError,
            SessionKey,
            WriterId,
            cells::{
                Cells,
                Previous,
            },
        },
    },
};

/// Most writers whose layers one document holds. Below the stack's own cap,
/// which also holds the document's other layers.
pub const MAX_LAYERS_PER_DOC: usize = 48;

/// Further layers past [`MAX_LAYERS_PER_DOC`] only the document's author and
/// the local peer may take, so a crowd of strangers cannot lock them out.
pub const RESERVED_LAYERS: usize = 8;

/// One opinion as composed: its key, its value, and the stamp of the state
/// that set it.
pub(crate) type Opinion = (SessionKey, Option<Vec<u8>>, u64);

/// What applying a tick or a layer changed.
#[derive(Default)]
pub(crate) struct Applied {
    pub(crate) set:     Vec<Opinion>,
    /// Keys that had an opinion and no longer do.
    pub(crate) cleared: Vec<SessionKey>,
    /// Whether the writer's layer exists, with the source among its sources.
    pub(crate) held:    bool,
}

/// A layer a departing source was the last to hold.
pub(crate) struct Orphan {
    pub(crate) doc:    DocId,
    pub(crate) space:  SpaceId,
    pub(crate) writer: WriterId,
    pub(crate) did:    Did,
}

/// The budget a writer's layer is charged to, and whether it may take one of
/// the [`RESERVED_LAYERS`].
pub(crate) struct Charge<'a> {
    pub(crate) quota:    &'a Arc<Quota>,
    pub(crate) reserved: bool,
}

/// The local writer's own tick on one document.
pub(crate) struct LocalTick<'a> {
    pub(crate) device: EndpointId,
    pub(crate) did:    &'a Did,
    pub(crate) doc:    DocId,
    pub(crate) space:  SpaceId,
    pub(crate) writes: &'a [(SessionKey, Option<Vec<u8>>)],
    pub(crate) clears: &'a [SessionKey],
}

/// Each change a tick made, with the value it replaced, so a refused tick
/// leaves nothing behind.
type Undo = Vec<(SessionKey, Previous)>;

struct Layer {
    space:      SpaceId,
    did:        Did,
    cells:      Cells,
    /// The newest state, which signs `cells`. `None` only while a local
    /// layer's first tick is being signed.
    state:      Option<SignedLayer>,
    sources:    HashSet<EndpointId>,
    bytes:      StockLease,
    _doc_lease: StockLease,
}

impl Layer {
    fn at(&self) -> Option<u64> {
        self.state.as_ref().map(|state| state.payload.at)
    }

    /// Applies a tick's clears, then its writes, charging the bytes the cells
    /// grow by. A key the tick also writes is not cleared.
    fn apply(
        &mut self,
        writes: &[(SessionKey, Option<Vec<u8>>)],
        clears: &[SessionKey],
        at: u64,
    ) -> Result<(Applied, Undo), SessionError> {
        let written = writes.iter().map(|(key, _)| key).collect::<HashSet<_>>();
        let mut applied = Applied {
            held: true,
            ..Applied::default()
        };
        let mut undo = Undo::new();
        for key in clears.iter().filter(|key| !written.contains(key)) {
            let old = self.cells.remove(key);
            if old != Previous::Absent {
                applied.cleared.push(key.clone());
                undo.push((key.clone(), old));
            }
        }
        for (key, value) in writes {
            let old = self.cells.set(key.clone(), value.clone());
            if old != Previous::Held(value.clone()) {
                applied.set.push((key.clone(), value.clone(), at));
            }
            undo.push((key.clone(), old));
        }
        if self.bytes.resize(self.cells.bytes()).is_err() {
            self.undo(undo);
            return Err(SessionError::QuotaExceeded);
        }
        Ok((applied, undo))
    }

    fn undo(&mut self, undo: Undo) {
        for (key, old) in undo.into_iter().rev() {
            match old {
                Previous::Held(value) => {
                    self.cells.set(key, value);
                }
                Previous::Absent => {
                    self.cells.remove(&key);
                }
            }
        }
        if self.bytes.resize(self.cells.bytes()).is_err() {
            tracing::debug!("session lease kept at its larger size");
        }
    }

    fn snapshot(&self) -> Option<LayerSnapshot> {
        let state = self.state.as_ref()?.signed.clone();
        let cells = self
            .cells
            .iter()
            .map(|(key, value)| SessionWrite {
                prim:  key.prim,
                name:  key.name.to_string(),
                value: value.map(<[u8]>::to_vec),
            })
            .collect();
        Some(LayerSnapshot { state, cells })
    }

    fn opinions(&self) -> Vec<Opinion> {
        let at = self.at().unwrap_or(0);
        self.cells
            .iter()
            .map(|(key, value)| (key, value.map(<[u8]>::to_vec), at))
            .collect()
    }
}

#[derive(Default)]
pub(crate) struct SessionLayers {
    docs:    HashMap<DocId, HashMap<WriterId, Layer>>,
    /// Layers whose tick this peer refused. Later ticks build on the refused
    /// one, so they are refused too until a whole layer replaces it.
    refused: HashSet<(DocId, WriterId)>,
}

impl SessionLayers {
    /// The space any writer's layer places `doc` in.
    pub(crate) fn space_of(&self, doc: DocId) -> Option<SpaceId> {
        self.docs
            .get(&doc)?
            .values()
            .next()
            .map(|layer| layer.space)
    }

    fn layer(&self, doc: DocId, writer: WriterId) -> Option<&Layer> {
        self.docs.get(&doc)?.get(&writer)
    }

    /// `writer`'s layer on `doc`, created empty if it may be.
    fn open(
        &mut self,
        writer: WriterId,
        did: &Did,
        doc: DocId,
        space: SpaceId,
        charge: &Charge<'_>,
    ) -> Result<&mut Layer, SessionError> {
        let layers = self.docs.entry(doc).or_default();
        if layers.values().any(|layer| layer.space != space) {
            return Err(SessionError::WrongSpace);
        }
        if !layers.contains_key(&writer) {
            let cap = MAX_LAYERS_PER_DOC + if charge.reserved { RESERVED_LAYERS } else { 0 };
            if layers.len() >= cap {
                return Err(SessionError::QuotaExceeded);
            }
            let lease = |stock, units| {
                charge
                    .quota
                    .lease(stock, units)
                    .map_err(|_| SessionError::QuotaExceeded)
            };
            let layer = Layer {
                space,
                did: did.clone(),
                cells: Cells::default(),
                state: None,
                sources: HashSet::new(),
                bytes: lease(Stock::SessionMemory, 0)?,
                _doc_lease: lease(Stock::Documents, 1)?,
            };
            layers.insert(writer, layer);
        }
        layers.get_mut(&writer).ok_or(SessionError::Unavailable)
    }

    /// Drops `writer`'s layer on `doc` if nothing states or holds it.
    fn prune(&mut self, doc: DocId, writer: WriterId) {
        let Some(layers) = self.docs.get_mut(&doc) else {
            return;
        };
        if layers
            .get(&writer)
            .is_some_and(|layer| layer.state.is_none() && layer.sources.is_empty())
        {
            layers.remove(&writer);
        }
        if layers.is_empty() {
            self.docs.remove(&doc);
        }
    }

    /// Applies a tick its writer sent over its own link, all or none, and
    /// records `source` as holding the layer.
    pub(crate) fn apply_tick(
        &mut self,
        source: EndpointId,
        tick: &ReceivedTick,
        charge: &Charge<'_>,
    ) -> Result<Applied, SessionError> {
        let writer = tick.state.writer();
        let state = &tick.state.payload;
        if self.refused.contains(&(state.doc, writer)) {
            return Err(SessionError::QuotaExceeded);
        }
        let layer = self.open(writer, &state.did, state.doc, state.space, charge)?;
        if layer.at().is_some_and(|at| at >= state.at) {
            return Err(SessionError::Stale);
        }
        let result = layer.apply(&tick.writes, &tick.clears, state.at);
        if result.is_ok() {
            layer.state = Some(tick.state.clone());
            layer.sources.insert(source);
        }
        match result {
            Ok((applied, _)) => Ok(applied),
            Err(err) => {
                if let Some(layers) = self.docs.get_mut(&state.doc) {
                    layers.remove(&writer);
                }
                self.prune(state.doc, writer);
                self.refused.insert((state.doc, writer));
                Err(err)
            }
        }
    }

    /// Applies the local writer's own tick, stamped past its layer's last
    /// state, and signs the state it leaves. A tick that cannot be signed is
    /// undone.
    pub(crate) fn write_local(
        &mut self,
        tick: &LocalTick<'_>,
        charge: &Charge<'_>,
        sign: impl FnOnce(LayerState) -> anyhow::Result<SignedLayer>,
    ) -> Result<(Applied, SignedLayer), SessionError> {
        let writer = WriterId::new(tick.device, tick.did);
        let layer = self.open(writer, tick.did, tick.doc, tick.space, charge)?;
        let at = clock::current_micros().max(layer.at().map_or(0, |at| at.saturating_add(1)));
        let result = match layer.apply(tick.writes, tick.clears, at) {
            Ok((applied, undo)) => {
                let payload = LayerState {
                    device: tick.device,
                    did: tick.did.clone(),
                    doc: tick.doc,
                    space: tick.space,
                    at,
                    root: layer.cells.root(),
                };
                match sign(payload) {
                    Ok(state) => {
                        layer.state = Some(state.clone());
                        layer.sources.insert(tick.device);
                        Ok((applied, state))
                    }
                    Err(err) => {
                        tracing::warn!(%err, "session state not signed");
                        layer.undo(undo);
                        Err(SessionError::Unsigned)
                    }
                }
            }
            Err(err) => Err(err),
        };
        if result.is_err() {
            self.prune(tick.doc, writer);
        }
        result
    }

    /// Replaces `writer`'s layer with a whole one a snapshot states, unless
    /// the one held is as new, and records `source` as holding it.
    pub(crate) fn replace_layer(
        &mut self,
        source: EndpointId,
        received: ReceivedLayer,
        charge: &Charge<'_>,
    ) -> Result<Applied, SessionError> {
        let writer = received.state.writer();
        let state = &received.state.payload;
        let (doc, space, at) = (state.doc, state.space, state.at);
        if let Some(held_at) = self.layer(doc, writer).and_then(Layer::at)
            && held_at >= at
        {
            if held_at == at
                && let Some(layer) = self.docs.get_mut(&doc).and_then(|l| l.get_mut(&writer))
            {
                layer.sources.insert(source);
            }
            return Ok(Applied {
                held: self
                    .layer(doc, writer)
                    .is_some_and(|l| l.sources.contains(&source)),
                ..Applied::default()
            });
        }
        let did = received.state.payload.did.clone();
        let layer = self.open(writer, &did, doc, space, charge)?;
        if layer.bytes.resize(received.cells.bytes()).is_err() {
            self.prune(doc, writer);
            return Err(SessionError::QuotaExceeded);
        }
        let mut applied = Applied {
            held: true,
            ..Applied::default()
        };
        for (key, _) in layer.cells.iter() {
            if received.cells.get(&key).is_none() {
                applied.cleared.push(key);
            }
        }
        for (key, value) in received.cells.iter() {
            if layer.cells.get(&key).map(Option::as_deref) != Some(value) {
                applied.set.push((key, value.map(<[u8]>::to_vec), at));
            }
        }
        layer.cells = received.cells;
        layer.state = Some(received.state);
        layer.sources.insert(source);
        self.refused.remove(&(doc, writer));
        Ok(applied)
    }

    /// Forgets the ticks refused from `device`, once it is gone.
    pub(crate) fn forget_refused(&mut self, device: EndpointId) {
        self.refused.retain(|(_, writer)| writer.device != device);
    }

    /// Writers holding a layer on `doc`, with the DID each layer names.
    pub(crate) fn layers(&self, doc: DocId) -> Vec<(WriterId, Did)> {
        self.docs.get(&doc).map_or_default(|layers| {
            layers
                .iter()
                .map(|(writer, layer)| (*writer, layer.did.clone()))
                .collect()
        })
    }

    /// Writers holding a layer on `doc`.
    pub(crate) fn writers(&self, doc: DocId) -> Vec<WriterId> {
        self.docs
            .get(&doc)
            .map_or_default(|layers| layers.keys().copied().collect())
    }

    /// `writer`'s opinions on `doc`.
    pub(crate) fn opinions(&self, writer: WriterId, doc: DocId) -> Vec<Opinion> {
        self.layer(doc, writer).map_or_default(Layer::opinions)
    }

    /// The space `writer`'s layer on `doc` is in, and the keys it states.
    pub(crate) fn keys_of(
        &self,
        writer: WriterId,
        doc: DocId,
    ) -> Option<(SpaceId, Vec<SessionKey>)> {
        let layer = self.layer(doc, writer)?;
        let keys = layer.cells.iter().map(|(key, _)| key).collect::<Vec<_>>();
        (!keys.is_empty()).then_some((layer.space, keys))
    }

    /// Every layer, for inspection.
    pub(crate) fn all(&self) -> Vec<(DocId, WriterId, Vec<Opinion>)> {
        self.docs
            .iter()
            .flat_map(|(doc, layers)| {
                layers
                    .iter()
                    .map(|(writer, layer)| (*doc, *writer, layer.opinions()))
            })
            .collect()
    }

    /// The peers `writer`'s layer on `doc` is held through.
    pub(crate) fn sources(&self, writer: WriterId, doc: DocId) -> Vec<EndpointId> {
        self.layer(doc, writer)
            .map_or_default(|layer| layer.sources.iter().copied().collect())
    }

    /// Every layer `device` writes, with each peer it is held through.
    pub(crate) fn sources_of_device(
        &self,
        device: EndpointId,
    ) -> Vec<(DocId, WriterId, EndpointId)> {
        let mut out = Vec::new();
        for (doc, layers) in &self.docs {
            for (writer, layer) in layers.iter().filter(|(writer, _)| writer.device == device) {
                out.extend(layer.sources.iter().map(|source| (*doc, *writer, *source)));
            }
        }
        out
    }

    /// Layers `source` alone holds, other than ones `me` writes.
    pub(crate) fn orphaned_by(&self, source: EndpointId, me: EndpointId) -> Vec<Orphan> {
        let mut out = Vec::new();
        for (doc, layers) in &self.docs {
            for (writer, layer) in layers {
                if writer.device != me
                    && layer.sources.len() == 1
                    && layer.sources.contains(&source)
                {
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

    /// Every layer `source` holds, as it would relay them.
    pub(crate) fn snapshot_of(&self, source: EndpointId) -> Vec<(DocId, SpaceId, LayerSnapshot)> {
        let mut out = Vec::new();
        for (doc, layers) in &self.docs {
            for layer in layers.values().filter(|l| l.sources.contains(&source)) {
                if let Some(snapshot) = layer.snapshot() {
                    out.push((*doc, layer.space, snapshot));
                }
            }
        }
        out
    }

    /// Records `source` as holding `writer`'s layer on `doc`. Answers whether
    /// the layer exists.
    pub(crate) fn add_source(&mut self, writer: WriterId, doc: DocId, source: EndpointId) -> bool {
        self.docs
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
        writer: WriterId,
        doc: DocId,
        source: EndpointId,
    ) -> bool {
        let Some(layers) = self.docs.get_mut(&doc) else {
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
            self.docs.remove(&doc);
        }
        true
    }
}
