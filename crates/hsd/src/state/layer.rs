use std::collections::HashMap;

use smol_str::SmolStr;

use crate::{
    id::PrimId,
    key,
    property::{
        Parent,
        Property,
    },
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
}

/// Which key of a prim an opinion is about.
///
/// Parent is not a property name here for the same reason it is a reserved key
/// in the format: it decides realization, so it settles the prim rather than
/// recomposing one value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum OpinionKey {
    Parent,
    Property(SmolStr),
    Slot(SmolStr),
}

/// One layer's opinions, keyed by prim.
#[derive(Debug, Default, Clone)]
pub(super) struct Layer(HashMap<PrimId, PrimOpinions>);

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
            out.extend(
                opinions
                    .slots()
                    .map(|(name, _)| (*prim, OpinionKey::Slot(name.clone()))),
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
    pub(super) fn take_parent(&mut self, prim: PrimId) -> Option<(Opinion<Parent>, Stamp)> {
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

    pub(super) fn take_slot(
        &mut self,
        prim: PrimId,
        name: &str,
    ) -> Option<(Opinion<Vec<u8>>, Stamp)> {
        let opinions = self.0.get_mut(&prim)?;
        let taken = opinions.take_slot(name);
        if opinions.is_empty() {
            self.0.remove(&prim);
        }
        taken
    }
}

/// What one document says about the prims of a document its `site` prim
/// references.
///
/// Durable in the referencing document, where it lives under `o/<site>/`, and
/// installed into the referenced document's state as its
/// [`LayerId::Override`] layer, which is where it beats the target's own
/// opinion.
#[derive(Debug, Default, Clone)]
pub struct Overrides(Layer);

impl Overrides {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.0.is_empty()
    }

    pub(super) const fn layer(&self) -> &Layer {
        &self.0
    }

    pub(super) const fn layer_mut(&mut self) -> &mut Layer {
        &mut self.0
    }

    /// Every opinion as the entry that carries it.
    ///
    /// A `Blocked` opinion is an explicit empty value rather than an absent
    /// key: absence falls through to the target's own value, which is the
    /// opposite of what blocking a key means.
    pub(super) fn entries(&self, site: PrimId) -> Vec<(String, Vec<u8>)> {
        let mut out = Vec::new();
        for (target, opinions) in self.0.prims() {
            if let Some((opinion, _)) = opinions.parent() {
                out.push((
                    key::override_key(site, target, key::PARENT),
                    opinion.value().map(Parent::encode).unwrap_or_default(),
                ));
            }
            for (name, opinion) in opinions.properties() {
                out.push((
                    key::override_key(site, target, name),
                    opinion.value().map(Property::encode).unwrap_or_default(),
                ));
            }
            for (name, opinion) in opinions.slots() {
                out.push((
                    key::override_key(site, target, name),
                    opinion.value().cloned().unwrap_or_default(),
                ));
            }
        }
        out
    }
}
