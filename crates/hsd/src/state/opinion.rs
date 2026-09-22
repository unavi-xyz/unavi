use std::collections::BTreeMap;

use smol_str::SmolStr;

use crate::{
    attributes::parent::ParentAttr,
    property::Property,
    state::entry::Stamp,
};

/// What one layer says about one key.
///
/// Absence from the layer's map is the third state — "no opinion" — and falls
/// through to the layer below. `Blocked` does not fall through: it states the
/// key is gone, so nothing weaker shows past it. Two states would leave "a
/// stronger layer removes a property the document set" inexpressible, since
/// absence already means "no opinion".
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

/// One layer's opinions about one prim, each stamped so a later write from any
/// peer wins and an older one is refused.
#[derive(Debug, Clone)]
pub(super) struct PrimOpinions {
    parent: Option<(Opinion<ParentAttr>, Stamp)>,
    /// Attributes, relationships and blobs share one map: a value's tag says
    /// which it is, so a name has exactly one kind and no tombstone has to
    /// guess between two.
    props:  BTreeMap<SmolStr, (Opinion<Property>, Stamp)>,
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

    pub(super) fn property(&self, name: &str) -> Option<&Opinion<Property>> {
        self.props.get(name).map(|(opinion, _)| opinion)
    }

    /// Every property key this layer holds an opinion about, `Blocked` ones
    /// included.
    pub(super) fn properties(&self) -> impl Iterator<Item = (&SmolStr, &Opinion<Property>)> {
        self.props
            .iter()
            .map(|(name, (opinion, _))| (name, opinion))
    }

    /// The properties this layer states a value for. A `Blocked` key is an
    /// opinion but not a value, so it is absent here and from the save set.
    pub(super) fn set_properties(&self) -> impl Iterator<Item = (&SmolStr, &Property)> {
        self.props
            .iter()
            .filter_map(|(name, (opinion, _))| opinion.value().map(|value| (name, value)))
    }

    pub(super) fn is_empty(&self) -> bool {
        self.parent.is_none() && self.props.is_empty()
    }

    /// Removes and answers the parent opinion, or `None` when this prim has
    /// none in this layer. An empty prim's entry is dropped by the layer's
    /// `take_*` wrappers, so a layer never lingers on prims it no longer
    /// says anything about.
    pub(super) const fn take_parent(&mut self) -> Option<(Opinion<ParentAttr>, Stamp)> {
        self.parent.take()
    }

    pub(super) fn take_property(&mut self, name: &str) -> Option<(Opinion<Property>, Stamp)> {
        self.props.remove(name)
    }

    /// Records an opinion, answering whether the write was accepted. A stamp
    /// older than the one already held is refused and changes nothing, which
    /// is what makes entries arriving out of order converge.
    pub(super) fn set_parent(&mut self, parent: Opinion<ParentAttr>, stamp: Stamp) -> bool {
        if self.parent.as_ref().is_some_and(|(_, old)| stamp < *old) {
            return false;
        }
        self.parent = Some((parent, stamp));
        true
    }

    pub(super) fn set_property(
        &mut self,
        name: &str,
        value: Opinion<Property>,
        stamp: Stamp,
    ) -> bool {
        match self.props.get_mut(name) {
            Some((_, old)) if stamp < *old => false,
            Some(slot) => {
                *slot = (value, stamp);
                true
            }
            None => {
                self.props.insert(SmolStr::new(name), (value, stamp));
                true
            }
        }
    }
}
