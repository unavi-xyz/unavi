//! Session state on the wire: the ticks a writer sends over its own link, and
//! the whole layers a snapshot states or relays.
//!
//! A writer signs only a [`LayerState`]: the root hash of its cells after each
//! tick. A tick's writes travel unsigned, since the link they arrive on proves
//! their writer. A relayed layer carries every cell with the newest signed
//! state, and the receiver checks the cells hash to its root.

use hsd::id::{
    DocId,
    PrimId,
};
use iroh::EndpointId;
use serde::{
    Deserialize,
    Serialize,
};
use unavi_identity::signed::{
    Signable,
    Signed,
};
use xdid::{
    core::did::Did,
    method::key::Signer,
};

use crate::{
    membership::SpaceId,
    replication::{
        frames,
        message::Rejected,
        session::{
            SessionKey,
            WriterId,
            cells::Cells,
        },
    },
};

/// Longest property name a session write may carry. A name is a key, and a
/// key nobody can read back is not worth storing.
pub const SESSION_NAME_MAX_BYTES: usize = 256;

/// Most writes, and most clears, one tick may carry.
pub const MAX_WRITES_PER_TICK: usize = 256;

/// Longest value a session write may carry.
pub const MAX_VALUE_BYTES: usize = 16 * 1024;

/// What a writer's cells on `doc` hash to as of `at`, signed by its DID.
#[derive(Serialize, Deserialize, Clone)]
pub struct LayerState {
    /// The device writing the layer. Only the DID signs it, so it names a
    /// layer of that DID's and proves nothing about the device.
    pub device: EndpointId,
    pub did:    Did,
    pub doc:    DocId,
    pub space:  SpaceId,
    /// Microseconds, as the writer's clock reads, and past its previous
    /// state's. The newest state of a layer wins.
    pub at:     u64,
    /// The [`Cells::root`] of the layer's cells.
    pub root:   [u8; 32],
}

impl Signable for LayerState {
    const SIGNING_CONTEXT: &'static str = "wired/space/layer";
}

/// One tick of the sender's own session opinions on one document, applied
/// whole so no peer draws half of it.
#[derive(Serialize, Deserialize, Clone)]
pub struct SessionTick {
    /// A `value` of `None` blocks the key, which is how a delete propagates.
    pub writes: Vec<SessionWrite>,
    /// Drops the writer's own opinions on these keys, so the layers beneath
    /// show through again. Sent after a commit makes them durable.
    pub clears: Vec<ClearedKey>,
    /// The layer's state once the tick is applied.
    pub state:  Signed<LayerState>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ClearedKey {
    pub prim: PrimId,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SessionWrite {
    pub prim:  PrimId,
    pub name:  String,
    pub value: Option<Vec<u8>>,
}

/// One writer's whole layer: its cells and the newest state signing them. A
/// layer too large for a frame is split by cells, each piece repeating the
/// state.
#[derive(Serialize, Deserialize, Clone)]
pub struct LayerSnapshot {
    pub state: Signed<LayerState>,
    pub cells: Vec<SessionWrite>,
}

/// A signed layer state with its payload decoded once.
#[derive(Clone)]
pub struct SignedLayer {
    pub signed:  Signed<LayerState>,
    pub payload: LayerState,
}

impl SignedLayer {
    /// Signs `payload` with the writer's DID key.
    pub fn sign(payload: LayerState, key: &impl Signer) -> anyhow::Result<Self> {
        Ok(Self {
            signed: payload.sign(key)?,
            payload,
        })
    }

    pub(super) fn decode(signed: Signed<LayerState>) -> Result<Self, Rejected> {
        let payload = signed.payload().map_err(|_| Rejected::Malformed)?;
        Ok(Self { signed, payload })
    }

    #[must_use]
    pub fn writer(&self) -> WriterId {
        WriterId::new(self.payload.device, &self.payload.did)
    }
}

/// A [`SessionTick`] that passed every bound.
pub struct ReceivedTick {
    pub writes: Vec<(SessionKey, Option<Vec<u8>>)>,
    pub clears: Vec<SessionKey>,
    pub state:  SignedLayer,
}

/// A [`LayerSnapshot`] that passed every bound. A relayed one is not yet
/// checked against its signature or root.
pub struct ReceivedLayer {
    pub state: SignedLayer,
    pub cells: Cells,
}

/// The bounds every tick is held to, by its writer before sending and by every
/// receiver. Parses the keys it names.
pub fn check_tick(
    writes: &[SessionWrite],
    clears: &[ClearedKey],
) -> Result<(Vec<(SessionKey, Option<Vec<u8>>)>, Vec<SessionKey>), Rejected> {
    if clears.len() > MAX_WRITES_PER_TICK || !frames::fits_one_tick(writes) {
        return Err(Rejected::TickTooLarge);
    }
    let writes = writes.iter().map(checked_write).collect::<Result<_, _>>()?;
    let clears = clears
        .iter()
        .map(|clear| key(clear.prim, &clear.name))
        .collect::<Result<_, _>>()?;
    Ok((writes, clears))
}

impl SessionTick {
    pub(super) fn check(self) -> Result<ReceivedTick, Rejected> {
        let (writes, clears) = check_tick(&self.writes, &self.clears)?;
        Ok(ReceivedTick {
            writes,
            clears,
            state: SignedLayer::decode(self.state)?,
        })
    }
}

impl LayerSnapshot {
    /// Checks a layer a snapshot states for `doc` in `space`.
    pub(super) fn check(self, doc: DocId, space: SpaceId) -> Result<ReceivedLayer, Rejected> {
        let state = SignedLayer::decode(self.state)?;
        if state.payload.doc != doc || state.payload.space != space {
            return Err(Rejected::Mismatched);
        }
        let mut cells = Cells::default();
        for write in &self.cells {
            let (key, value) = checked_write(write)?;
            cells.set(key, value);
        }
        Ok(ReceivedLayer { state, cells })
    }
}

impl ReceivedLayer {
    /// Folds in another piece of the same layer: its cells when it states the
    /// same state, or the piece itself when it states a newer one.
    pub(super) fn merge(&mut self, piece: Self) {
        if piece.state.payload.at > self.state.payload.at {
            *self = piece;
        } else if piece.state.payload.at == self.state.payload.at
            && piece.state.payload.root == self.state.payload.root
        {
            for (key, value) in piece.cells.iter() {
                self.cells.set(key, value.map(<[u8]>::to_vec));
            }
        }
    }

    /// Whether the cells hash to the root the state signs.
    pub fn matches_root(&mut self) -> bool {
        self.cells.root() == self.state.payload.root
    }
}

fn checked_write(write: &SessionWrite) -> Result<(SessionKey, Option<Vec<u8>>), Rejected> {
    if write
        .value
        .as_ref()
        .is_some_and(|value| value.len() > MAX_VALUE_BYTES)
    {
        return Err(Rejected::ValueTooLarge);
    }
    Ok((key(write.prim, &write.name)?, write.value.clone()))
}

fn key(prim: PrimId, name: &str) -> Result<SessionKey, Rejected> {
    if name.len() > SESSION_NAME_MAX_BYTES {
        return Err(Rejected::NameTooLong);
    }
    let name = name.parse().map_err(|_| Rejected::Malformed)?;
    Ok(SessionKey { prim, name })
}
