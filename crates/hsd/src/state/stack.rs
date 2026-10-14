//! The ordered layers a document resolves from, and the masks deciding what
//! each may contribute.

use std::{
    collections::HashSet,
    fmt::{
        Debug,
        Display,
        Formatter,
    },
    sync::Arc,
};

use smol_str::SmolStr;
use thiserror::Error;

use crate::{
    attributes::parent::ParentAttr,
    id::PrimId,
    property::{
        name::PropName,
        value::Value,
    },
    state::{
        HsdState,
        layer::{
            Layer,
            OpinionKey,
        },
    },
};

/// Most layers one stack holds.
pub const MAX_LAYERS: usize = 64;

/// Names a layer within one stack. Whoever builds the stack chooses it.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LayerId(SmolStr);

impl LayerId {
    /// The document's own layer, at the bottom of every stack.
    pub const BASE: Self = Self::from_static("base");

    #[must_use]
    pub fn new(name: impl Into<SmolStr>) -> Self {
        Self(name.into())
    }

    #[must_use]
    pub const fn from_static(name: &'static str) -> Self {
        Self(SmolStr::new_static(name))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Debug for LayerId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "LayerId({})", self.0)
    }
}

impl Display for LayerId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// How a layer takes a write on a key it already holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Merge {
    /// The write replaces what is held. A layer projected from a store holds
    /// the winners the store already chose.
    Replace,
    /// A write stamped older than the one held is refused, since live writes
    /// arrive in any order.
    Latest,
}

/// One opinion a mask is asked about.
#[derive(Debug, Clone, Copy)]
pub enum MaskKey<'a> {
    /// The prim's parent. `None` blocks it.
    Parent(Option<ParentAttr>),
    /// A property. A `None` value blocks it.
    Property {
        name:    &'a PropName,
        value:   Option<&'a Value>,
        /// What the layers beneath this one resolve the key to, so a layer
        /// can be admitted only where it fills a gap.
        beneath: Option<&'a Value>,
    },
}

type MaskFn = dyn Fn(PrimId, MaskKey<'_>) -> bool + Send + Sync;

/// Which of a layer's opinions take part in resolution. A masked-out opinion
/// resolves as if the layer held none.
#[derive(Clone, Default)]
pub struct Mask(Option<Arc<MaskFn>>);

impl Mask {
    /// Every opinion takes part.
    #[must_use]
    pub const fn all() -> Self {
        Self(None)
    }

    #[must_use]
    pub fn new(admits: impl Fn(PrimId, MaskKey<'_>) -> bool + Send + Sync + 'static) -> Self {
        Self(Some(Arc::new(admits)))
    }

    #[must_use]
    pub fn admits(&self, prim: PrimId, key: MaskKey<'_>) -> bool {
        self.0.as_ref().is_none_or(|admits| admits(prim, key))
    }

    #[must_use]
    pub const fn is_all(&self) -> bool {
        self.0.is_none()
    }
}

impl Debug for Mask {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(if self.0.is_some() {
            "Mask(..)"
        } else {
            "Mask::all"
        })
    }
}

/// One layer in a stack.
#[derive(Debug, Clone)]
pub(super) struct Stacked {
    pub(super) id:    LayerId,
    pub(super) layer: Layer,
    pub(super) merge: Merge,
    pub(super) mask:  Mask,
}

impl Stacked {
    pub(super) fn base() -> Self {
        Self {
            id:    LayerId::BASE,
            layer: Layer::default(),
            merge: Merge::Replace,
            mask:  Mask::all(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum StackError {
    #[error("the stack already holds {MAX_LAYERS} layers")]
    Full,
    #[error("no layer {0} in the stack")]
    Unknown(LayerId),
    #[error("layer {0} is already in the stack")]
    Duplicate(LayerId),
    #[error("the base layer cannot be added or removed")]
    Base,
    #[error("an order must name every layer above the base exactly once")]
    NotAnOrder,
}

impl HsdState {
    /// The stack's layer ids, weakest first, the base among them.
    pub fn layers(&self) -> impl Iterator<Item = &LayerId> {
        self.stack.iter().map(|stacked| &stacked.id)
    }

    /// The opinions `id` holds.
    #[must_use]
    pub fn layer(&self, id: &LayerId) -> Option<&Layer> {
        self.position(id).map(|at| &self.stack[at].layer)
    }

    /// Adds an empty layer above every other.
    pub fn add_layer(&mut self, id: LayerId, merge: Merge, mask: Mask) -> Result<(), StackError> {
        if id == LayerId::BASE {
            return Err(StackError::Base);
        }
        if self.position(&id).is_some() {
            return Err(StackError::Duplicate(id));
        }
        if self.stack.len() >= MAX_LAYERS {
            return Err(StackError::Full);
        }
        self.stack.push(Stacked {
            id,
            layer: Layer::default(),
            merge,
            mask,
        });
        Ok(())
    }

    /// Removes `id`, re-resolving every key it held, and answers what it held.
    /// A write target removed falls back to the base.
    pub fn remove_layer(&mut self, id: &LayerId) -> Result<Layer, StackError> {
        let at = self.layer_position(id)?;
        if at == 0 {
            return Err(StackError::Base);
        }
        let removed = self.stack.remove(at);
        if self.edit_target == *id {
            self.edit_target = LayerId::BASE;
        }
        self.settle_keys(removed.layer.keys());
        for prim in removed.layer.prims() {
            self.forget_if_unstated(prim);
        }
        Ok(removed.layer)
    }

    /// Replaces what `id` holds, re-resolving every key either version held.
    pub fn replace_layer(&mut self, id: &LayerId, layer: Layer) -> Result<(), StackError> {
        let at = self.layer_position(id)?;
        let mut keys = self.stack[at]
            .layer
            .keys()
            .into_iter()
            .collect::<HashSet<_>>();
        keys.extend(layer.keys());
        let old = std::mem::replace(&mut self.stack[at].layer, layer);
        self.settle_keys(keys);
        for prim in old.prims() {
            self.forget_if_unstated(prim);
        }
        Ok(())
    }

    /// Changes which of `id`'s opinions take part, re-resolving each.
    pub fn set_mask(&mut self, id: &LayerId, mask: Mask) -> Result<(), StackError> {
        let at = self.layer_position(id)?;
        self.stack[at].mask = mask;
        let keys = self.stack[at].layer.keys();
        self.settle_keys(keys);
        Ok(())
    }

    /// Orders the layers above the base, weakest first. `order` names each of
    /// them exactly once.
    pub fn reorder(&mut self, order: &[LayerId]) -> Result<(), StackError> {
        let above = self.stack.len() - 1;
        let named = order.iter().collect::<HashSet<_>>();
        if order.len() != above
            || named.len() != above
            || named.contains(&LayerId::BASE)
            || order.iter().any(|id| self.position(id).is_none())
        {
            return Err(StackError::NotAnOrder);
        }
        if self.stack[1..]
            .iter()
            .map(|stacked| &stacked.id)
            .eq(order.iter())
        {
            return Ok(());
        }

        let mut rest = self.stack.split_off(1);
        let mut keys = HashSet::new();
        for id in order {
            let Some(at) = rest.iter().position(|stacked| stacked.id == *id) else {
                return Err(StackError::NotAnOrder);
            };
            let stacked = rest.swap_remove(at);
            keys.extend(stacked.layer.keys());
            self.stack.push(stacked);
        }
        self.settle_keys(keys);
        Ok(())
    }

    /// Which layer plain writes go to.
    #[must_use]
    pub const fn edit_target(&self) -> &LayerId {
        &self.edit_target
    }

    pub fn set_edit_target(&mut self, id: LayerId) -> Result<(), StackError> {
        self.layer_position(&id)?;
        self.edit_target = id;
        Ok(())
    }

    /// Where plain writes go. A target always names a layer in the stack.
    pub(super) fn edit_position(&self) -> usize {
        self.position(&self.edit_target).unwrap_or(0)
    }

    pub(super) fn position(&self, id: &LayerId) -> Option<usize> {
        self.stack.iter().position(|stacked| stacked.id == *id)
    }

    pub(super) fn layer_position(&self, id: &LayerId) -> Result<usize, StackError> {
        self.position(id)
            .ok_or_else(|| StackError::Unknown(id.clone()))
    }

    pub(super) fn settle_keys(&mut self, keys: impl IntoIterator<Item = (PrimId, OpinionKey)>) {
        for (prim, key) in keys {
            match key {
                OpinionKey::Parent => self.settle_parent(prim),
                OpinionKey::Property(name) => self.settle_property(prim, &name),
            }
        }
    }
}
