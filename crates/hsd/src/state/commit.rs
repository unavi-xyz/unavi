use bytes::Bytes;

use crate::{
    attributes::{
        parent::ParentAttr,
        reference::LayerKey,
    },
    id::PrimId,
    key,
    property::{
        Property,
        name::PropName,
    },
    state::{
        CommitTarget,
        HsdState,
        StateError,
        entry::{
            Entry,
            Stamp,
        },
        layer::{
            Layer,
            OpinionKey,
        },
        opinion::Opinion,
        stack::{
            LayerId,
            Merge,
        },
    },
};

impl HsdState {
    /// Moves each key's strongest opinion among the `from` layers into
    /// `target`, stamped as a new local write, and clears the key from every
    /// `from` layer. A key no `from` layer holds is skipped.
    ///
    /// Answers the entries the target's store must hold, none for
    /// [`CommitTarget::Layer`]. An empty value deletes its key. A prim
    /// committed as removed from the base takes its base properties and
    /// reference layers with it.
    pub fn commit(
        &mut self,
        from: &[LayerId],
        target: &CommitTarget,
        props: &[(PrimId, PropName)],
    ) -> Result<Vec<Entry>, StateError> {
        let layer = match target {
            CommitTarget::Base => 0,
            CommitTarget::Override { layer, .. } | CommitTarget::Layer(layer) => {
                self.layer_position(layer)?
            }
        };
        let mut sources = from
            .iter()
            .map(|id| self.layer_position(id))
            .collect::<Result<Vec<_>, _>>()?;
        sources.retain(|at| *at != layer);
        sources.sort_unstable_by(|a, b| b.cmp(a));

        let mut writes = Vec::new();
        for (prim, name) in props {
            let is_parent = *name == ParentAttr::NAME;
            let promoted = if is_parent {
                self.commit_parent(&sources, layer, *prim)
            } else {
                self.commit_property(&sources, layer, *prim, name)
            };
            let Some((value, timestamp)) = promoted else {
                continue;
            };
            let removes_prim = is_parent && value.is_empty();

            let key = match target {
                CommitTarget::Base => key::Key::prop(*prim, name).to_string(),
                CommitTarget::Override { site, .. } => LayerKey {
                    target: *prim,
                    name:   name.clone(),
                }
                .key(*site),
                CommitTarget::Layer(_) => continue,
            };
            writes.push(Entry {
                key,
                value,
                timestamp,
            });

            if removes_prim && *target == CommitTarget::Base {
                self.drop_base_prim(*prim, timestamp, &mut writes);
            }
        }
        Ok(writes)
    }

    /// Takes every property and reference layer the base layer states for a
    /// removed prim, answering a deletion for each.
    fn drop_base_prim(&mut self, prim: PrimId, timestamp: u64, writes: &mut Vec<Entry>) {
        let names = self.stack[0].layer.get(prim).map_or_default(|opinions| {
            opinions
                .properties()
                .map(|(name, _)| name.clone())
                .collect::<Vec<_>>()
        });
        for name in names {
            self.stack[0].layer.take_property(prim, &name);
            self.settle_property(prim, &name);
            writes.push(Entry::new(
                key::Key::prop(prim, &name).to_string(),
                Vec::new(),
                timestamp,
            ));
        }

        let Some(references) = self.references.remove(&prim) else {
            return;
        };
        for (target, opinion) in references.keys() {
            let name = match opinion {
                OpinionKey::Parent => ParentAttr::NAME,
                OpinionKey::Property(name) => name,
            };
            writes.push(Entry::new(
                LayerKey { target, name }.key(prim),
                Vec::new(),
                timestamp,
            ));
        }
    }

    /// The promoted entry value and timestamp. `Blocked` encodes empty.
    fn commit_property(
        &mut self,
        sources: &[usize],
        target: usize,
        prim: PrimId,
        name: &PropName,
    ) -> Option<(Bytes, u64)> {
        let (opinion, _) =
            self.take_from(sources, prim, |layer, id| layer.take_property(id, name))?;
        let timestamp = self.local_property_time(target, prim, name);
        let (value, stamp) = opinion.value().map_or_else(
            || (Vec::new(), Stamp::new(timestamp, &[])),
            |property| (property.encode(), Stamp::of_property(timestamp, property)),
        );
        let stacked = &mut self.stack[target];
        let opinions = stacked.layer.entry(prim);
        match stacked.merge {
            Merge::Replace => opinions.replace_property(name, opinion, stamp),
            Merge::Latest => {
                opinions.set_property(name, opinion, stamp);
            }
        }
        self.settle_property(prim, name);
        Some((value.into(), timestamp))
    }

    fn commit_parent(
        &mut self,
        sources: &[usize],
        target: usize,
        prim: PrimId,
    ) -> Option<(Bytes, u64)> {
        let (opinion, _) = self.take_from(sources, prim, Layer::take_parent)?;
        let parent = opinion.value().copied();
        let stamp = self.local_parent_stamp(target, prim, parent);
        let stacked = &mut self.stack[target];
        let opinions = stacked.layer.entry(prim);
        match stacked.merge {
            Merge::Replace => opinions.replace_parent(opinion, stamp),
            Merge::Latest => {
                opinions.set_parent(opinion, stamp);
            }
        }
        self.settle_parent(prim);
        Some((ParentAttr::to_wire(parent).into(), stamp.timestamp))
    }

    /// Takes the strongest opinion on a key among `sources`, ordered
    /// strongest first, and clears the key from each, so no weaker source
    /// resolves above the target.
    fn take_from<T>(
        &mut self,
        sources: &[usize],
        prim: PrimId,
        mut take: impl FnMut(&mut Layer, PrimId) -> Option<(Opinion<T>, Stamp)>,
    ) -> Option<(Opinion<T>, Stamp)> {
        let mut taken = None;
        for &at in sources {
            if let Some(opinion) = take(&mut self.stack[at].layer, prim) {
                taken = taken.or(Some(opinion));
            }
        }
        taken
    }
}
