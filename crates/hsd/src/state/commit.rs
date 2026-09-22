//! Promoting live opinions into a durable or session layer.

use smol_str::SmolStr;

use crate::{
    attributes::{
        Attribute,
        parent::ParentAttr,
        reference,
    },
    id::PrimId,
    key,
    property::Property,
    state::{
        CommitTarget,
        HsdState,
        entry::{
            Entry,
            Stamp,
        },
        layer::{
            Layer,
            LayerId,
        },
        opinion::Opinion,
    },
};

impl HsdState {
    /// Promotes each named key's strongest live opinion into `target` and
    /// drops it from every live layer, so the key resolves from the target
    /// alone.
    ///
    /// A name with no live opinion is skipped: nothing to keep, so nothing
    /// changes. The composed value never moves — the opinion travels whole,
    /// taking its stamp with it — so a promotion emits no event; what a later
    /// save writes is what changes.
    ///
    /// Answers the entries a *referencing* document has to hold for the
    /// promotion to survive, which is empty for every target but
    /// [`CommitTarget::Override`]: an override is durable in the document that
    /// states it, not in this one.
    pub fn commit(&mut self, target: CommitTarget, props: &[(PrimId, SmolStr)]) -> Vec<Entry> {
        let (layer, site) = match target {
            CommitTarget::Document => (LayerId::Document, None),
            CommitTarget::Override { site } => (LayerId::Override, Some(site)),
            CommitTarget::Session => (LayerId::Session, None),
        };

        let mut entries = Vec::new();
        for (prim, name) in props {
            if !key::is_valid_name(name) {
                continue;
            }
            let promoted = match name.as_str() {
                ParentAttr::KEY => self.commit_parent(layer, *prim),
                name => self.commit_property(layer, *prim, name),
            };
            if let Some(site) = site
                && let Some((value, timestamp)) = promoted
            {
                entries.push(Entry {
                    key: reference::layer_key(site, *prim, name),
                    value,
                    timestamp,
                });
            }
        }
        entries
    }

    /// The bytes and timestamp a promotion carried, or `None` where there was
    /// no live opinion to promote. A `Blocked` opinion carries no value, and
    /// an empty one is how the format spells it.
    fn commit_property(
        &mut self,
        target: LayerId,
        prim: PrimId,
        name: &str,
    ) -> Option<(Vec<u8>, u64)> {
        let (opinion, stamp) = self.take_live_property(prim, name)?;
        let value = opinion.value().map(Property::encode).unwrap_or_default();
        self.layer(target)
            .entry(prim)
            .set_property(name, opinion, stamp);
        // Recompute unconditionally: the refusal of an older stamp above can
        // leave the cache stale, and a promotion must read as whatever the
        // stack now says.
        self.settle_property(prim, name);
        Some((value, stamp.timestamp))
    }

    /// Promoting a parent opinion re-settles the prim, exactly as any parent
    /// write would: realization, sibling index and the subtree beneath it all
    /// answer to where the key resolves.
    fn commit_parent(&mut self, target: LayerId, prim: PrimId) -> Option<(Vec<u8>, u64)> {
        let (opinion, stamp) = self.take_live_parent(prim)?;
        let value = ParentAttr::to_wire(opinion.value().copied());
        self.layer(target).entry(prim).set_parent(opinion, stamp);
        self.settle_parent(prim);
        Some((value, stamp.timestamp))
    }

    /// The strongest live opinion on a property, removed from every live
    /// layer. A stronger layer's take leaves a weaker one's shadowed opinion
    /// behind, so both are cleared.
    fn take_live_property(
        &mut self,
        prim: PrimId,
        name: &str,
    ) -> Option<(Opinion<Property>, Stamp)> {
        self.take_live(prim, |layer, id| layer.take_property(id, name))
    }

    fn take_live_parent(&mut self, prim: PrimId) -> Option<(Opinion<ParentAttr>, Stamp)> {
        self.take_live(prim, Layer::take_parent)
    }

    /// Takes the strongest live opinion on a key, clearing every live layer of
    /// whatever it holds on it: a weaker layer's shadowed opinion would
    /// otherwise resolve above the target the strongest one was promoted into.
    fn take_live<T>(
        &mut self,
        prim: PrimId,
        mut take: impl FnMut(&mut Layer, PrimId) -> Option<(Opinion<T>, Stamp)>,
    ) -> Option<(Opinion<T>, Stamp)> {
        let mut taken = None;
        for id in [LayerId::Session, LayerId::Runtime] {
            if let Some(layer) = self.layers.get_mut(id.idx())
                && let Some(opinion) = take(layer, prim)
            {
                taken = taken.or(Some(opinion));
            }
        }
        taken
    }
}
