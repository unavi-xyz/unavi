use crate::{
    id::PrimId,
    property::{
        Property,
        name::PropName,
    },
    schema::{
        parent::ParentAttr,
        reference::LayerKey,
    },
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
    /// Moves each key's strongest live opinion into `target`, stamped as a
    /// new local write, and clears the key from every live layer. A key with
    /// no live opinion is skipped. The composed value does not change, so no
    /// event is emitted.
    ///
    /// Answers the entries the referencing document must hold, which are
    /// empty unless `target` is [`CommitTarget::Override`].
    pub fn commit(&mut self, target: CommitTarget, props: &[(PrimId, PropName)]) -> Vec<Entry> {
        let (layer, site) = match target {
            CommitTarget::Document => (LayerId::Document, None),
            CommitTarget::Override { site } => (LayerId::Override, Some(site)),
            CommitTarget::Session => (LayerId::Session, None),
        };

        let mut entries = Vec::new();
        for (prim, name) in props {
            let promoted = if *name == ParentAttr::NAME {
                self.commit_parent(layer, *prim)
            } else {
                self.commit_property(layer, *prim, name)
            };
            if let Some(site) = site
                && let Some((value, timestamp)) = promoted
            {
                entries.push(Entry {
                    key: LayerKey {
                        target: *prim,
                        name:   name.clone(),
                    }
                    .key(site),
                    value,
                    timestamp,
                });
            }
        }
        entries
    }

    /// The promoted entry value and timestamp. `Blocked` encodes empty.
    fn commit_property(
        &mut self,
        target: LayerId,
        prim: PrimId,
        name: &PropName,
    ) -> Option<(Vec<u8>, u64)> {
        let (opinion, _) = self.take_live(prim, |layer, id| layer.take_property(id, name))?;
        let timestamp = self.local_property_time(target, prim, name);
        let (value, stamp) = opinion.value().map_or_else(
            || (Vec::new(), Stamp::new(timestamp, &[])),
            |property| (property.encode(), Stamp::of_property(timestamp, property)),
        );
        self.layer_mut(target)
            .entry(prim)
            .set_property(name, opinion, stamp);
        self.settle_property(prim, name);
        Some((value, timestamp))
    }

    fn commit_parent(&mut self, target: LayerId, prim: PrimId) -> Option<(Vec<u8>, u64)> {
        let (opinion, _) = self.take_live(prim, Layer::take_parent)?;
        let parent = opinion.value().copied();
        let stamp = self.local_parent_stamp(target, prim, parent);
        self.layer_mut(target)
            .entry(prim)
            .set_parent(opinion, stamp);
        self.settle_parent(prim);
        Some((ParentAttr::to_wire(parent), stamp.timestamp))
    }

    /// Takes the strongest live opinion on a key and clears the key from every
    /// live layer, so no weaker live opinion resolves above the target.
    fn take_live<T>(
        &mut self,
        prim: PrimId,
        mut take: impl FnMut(&mut Layer, PrimId) -> Option<(Opinion<T>, Stamp)>,
    ) -> Option<(Opinion<T>, Stamp)> {
        let mut taken = None;
        for id in [LayerId::Session, LayerId::Runtime] {
            if let Some(opinion) = take(self.layer_mut(id), prim) {
                taken = taken.or(Some(opinion));
            }
        }
        taken
    }
}
