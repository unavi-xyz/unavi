use std::collections::HashMap;

use crate::{
    attributes::parent::ParentAttr,
    id::PrimId,
    property::{
        name::PropName,
        value::Value,
    },
    state::{
        entry::Stamp,
        opinion::{
            Opinion,
            PrimOpinions,
        },
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum OpinionKey {
    Parent,
    Property(PropName),
}

/// One layer's opinions, keyed by prim.
#[derive(Debug, Default, Clone)]
pub struct Layer(HashMap<PrimId, PrimOpinions>);

impl Layer {
    pub(super) fn get(&self, prim: PrimId) -> Option<&PrimOpinions> {
        self.0.get(&prim)
    }

    pub(super) fn prims(&self) -> Vec<PrimId> {
        self.0.keys().copied().collect()
    }

    pub(super) fn keys(&self) -> Vec<(PrimId, OpinionKey)> {
        let mut out = Vec::new();
        for (prim, opinions) in &self.0 {
            if opinions.parent().is_some() {
                out.push((*prim, OpinionKey::Parent));
            }
            out.extend(
                opinions
                    .properties()
                    .map(|(name, _)| (*prim, OpinionKey::Property(name.clone()))),
            );
        }
        out
    }

    pub(super) fn entry(&mut self, prim: PrimId) -> &mut PrimOpinions {
        self.0.entry(prim).or_insert_with(PrimOpinions::new)
    }

    pub(super) fn take_parent(&mut self, prim: PrimId) -> Option<(Opinion<ParentAttr>, Stamp)> {
        let opinions = self.0.get_mut(&prim)?;
        let taken = opinions.take_parent();
        if opinions.is_empty() {
            self.0.remove(&prim);
        }
        taken
    }

    pub(super) fn take_property(
        &mut self,
        prim: PrimId,
        name: &PropName,
    ) -> Option<(Opinion<Value>, Stamp)> {
        let opinions = self.0.get_mut(&prim)?;
        let taken = opinions.take_property(name);
        if opinions.is_empty() {
            self.0.remove(&prim);
        }
        taken
    }

    pub(super) fn remove(&mut self, prim: PrimId) {
        self.0.remove(&prim);
    }
}
