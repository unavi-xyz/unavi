use std::collections::BTreeMap;

use crate::{
    property::{
        name::PropName,
        value::Value,
    },
    schema::parent::ParentAttr,
    state::entry::Stamp,
};

/// What one layer says about one key. A layer holding no opinion falls
/// through to the layer below, and `Blocked` hides every weaker layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Opinion<T> {
    Set(T),
    Blocked,
}

impl<T> Opinion<T> {
    pub(super) const fn value(&self) -> Option<&T> {
        match self {
            Self::Set(value) => Some(value),
            Self::Blocked => None,
        }
    }
}

impl<T> From<Option<T>> for Opinion<T> {
    fn from(value: Option<T>) -> Self {
        value.map_or(Self::Blocked, Self::Set)
    }
}

/// One layer's opinions about one prim, each stamped so an older write is
/// refused.
#[derive(Debug, Clone)]
pub(super) struct PrimOpinions {
    parent: Option<(Opinion<ParentAttr>, Stamp)>,
    props:  BTreeMap<PropName, (Opinion<Value>, Stamp)>,
}

impl PrimOpinions {
    pub(super) const fn new() -> Self {
        Self {
            parent: None,
            props:  BTreeMap::new(),
        }
    }

    pub(super) const fn parent(&self) -> Option<&(Opinion<ParentAttr>, Stamp)> {
        self.parent.as_ref()
    }

    pub(super) fn property(&self, name: &PropName) -> Option<&Opinion<Value>> {
        self.props.get(name).map(|(opinion, _)| opinion)
    }

    pub(super) fn property_stamp(&self, name: &PropName) -> Option<Stamp> {
        self.props.get(name).map(|(_, stamp)| *stamp)
    }

    /// `Blocked` opinions included.
    pub(super) fn properties(&self) -> impl Iterator<Item = (&PropName, &Opinion<Value>)> {
        self.props
            .iter()
            .map(|(name, (opinion, _))| (name, opinion))
    }

    /// `Blocked` opinions excluded.
    pub(super) fn set_properties(&self) -> impl Iterator<Item = (&PropName, &Value)> {
        self.props
            .iter()
            .filter_map(|(name, (opinion, _))| opinion.value().map(|value| (name, value)))
    }

    pub(super) fn is_empty(&self) -> bool {
        self.parent.is_none() && self.props.is_empty()
    }

    pub(super) const fn take_parent(&mut self) -> Option<(Opinion<ParentAttr>, Stamp)> {
        self.parent.take()
    }

    pub(super) fn take_property(&mut self, name: &PropName) -> Option<(Opinion<Value>, Stamp)> {
        self.props.remove(name)
    }

    /// Answers whether the write was accepted. A stamp older than the one
    /// held is refused.
    pub(super) fn set_parent(&mut self, parent: Opinion<ParentAttr>, stamp: Stamp) -> bool {
        if self.parent.as_ref().is_some_and(|(_, old)| stamp < *old) {
            return false;
        }
        self.parent = Some((parent, stamp));
        true
    }

    pub(super) fn set_property(
        &mut self,
        name: &PropName,
        value: Opinion<Value>,
        stamp: Stamp,
    ) -> bool {
        match self.props.get_mut(name) {
            Some((_, old)) if stamp < *old => false,
            Some(slot) => {
                *slot = (value, stamp);
                true
            }
            None => {
                self.props.insert(name.clone(), (value, stamp));
                true
            }
        }
    }
}
