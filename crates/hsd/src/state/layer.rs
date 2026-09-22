use std::collections::HashMap;

use smol_str::SmolStr;

use crate::{
    attributes::parent::ParentAttr,
    id::PrimId,
    property::Property,
    state::{
        entry::Stamp,
        opinion::{
            Opinion,
            PrimOpinions,
        },
    },
};

/// Strength order: a later variant's opinion wins over an earlier one's.
///
/// `Document` is what a keyholder wrote and what a save writes back.
/// `Override` is what the document referencing this one says about its prims,
/// which is durable in *that* document; it beats this document's own opinion
/// and loses to anything live. `Runtime` is what this peer's scripts said this
/// session, and is replicated by nothing. `Session` is what a commit falls
/// back to when the caller holds no durable key; it resolves above `Runtime`
/// and, like it, is replicated by nothing yet — replication of session
/// opinions is the session-layer workstream's.
///
/// The reference chain is one rung rather than a depth index, because an
/// override key names a site prim and a target prim and so reaches exactly one
/// hop. A room that references a couch that references a cushion cannot state
/// an opinion about the cushion at all, so there is no second rung to hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum LayerId {
    Document,
    Override,
    Runtime,
    Session,
}

impl LayerId {
    pub(super) const ALL: [Self; 4] =
        [Self::Document, Self::Override, Self::Runtime, Self::Session];

    /// This seat's index in [`HsdState::layers`](crate::state::HsdState),
    /// which is what carries its strength: weakest first.
    pub(super) const fn idx(self) -> usize {
        self as usize
    }
}

/// Which key of a prim an opinion is about.
///
/// Parent is not a property name here for the same reason it is a reserved
/// key in the format: it decides realization, so it settles the prim rather
/// than recomposing one value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum OpinionKey {
    Parent,
    Property(SmolStr),
}

/// One layer's opinions, keyed by prim.
///
/// Public because a document's reference layers are plain `Layer`s: what it
/// says about the prims of the documents it references, and what a document
/// referencing it installs into the referenced document's `Override` seat.
#[derive(Debug, Default, Clone)]
pub struct Layer(HashMap<PrimId, PrimOpinions>);

impl Layer {
    pub(super) fn get(&self, prim: PrimId) -> Option<&PrimOpinions> {
        self.0.get(&prim)
    }

    /// Every key this layer holds an opinion about, which is what installing a
    /// replacement layer has to recompose.
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

    pub(super) fn prims(&self) -> impl Iterator<Item = (PrimId, &PrimOpinions)> {
        self.0.iter().map(|(prim, opinions)| (*prim, opinions))
    }

    /// Removes this layer's opinion on a key, answering what it was. Used by
    /// `commit`, which lifts a live opinion into its target and must then drop
    /// it from every live layer so the key resolves from the target alone.
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
        name: &str,
    ) -> Option<(Opinion<Property>, Stamp)> {
        let opinions = self.0.get_mut(&prim)?;
        let taken = opinions.take_property(name);
        if opinions.is_empty() {
            self.0.remove(&prim);
        }
        taken
    }
}
