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
        entry::{
            Entry,
            Stamp,
        },
        layer::{
            Layer,
            LayerId,
            OpinionKey,
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
    /// Answers the entries the target's store must hold, none for
    /// [`CommitTarget::Session`]. An empty value deletes its key. A prim
    /// committed as removed from the document takes its document properties
    /// and reference layers with it.
    pub fn commit(&mut self, target: CommitTarget, props: &[(PrimId, PropName)]) -> Vec<Entry> {
        let layer = match target {
            CommitTarget::Document => LayerId::Document,
            CommitTarget::Override { .. } => LayerId::Override,
            CommitTarget::Session => LayerId::Session,
        };

        let mut writes = Vec::new();
        for (prim, name) in props {
            let is_parent = *name == ParentAttr::NAME;
            let promoted = if is_parent {
                self.commit_parent(layer, *prim)
            } else {
                self.commit_property(layer, *prim, name)
            };
            let Some((value, timestamp)) = promoted else {
                continue;
            };
            let removes_prim = is_parent && value.is_empty();

            let key = match target {
                CommitTarget::Document => key::Key::prop(*prim, name).to_string(),
                CommitTarget::Override { site } => LayerKey {
                    target: *prim,
                    name:   name.clone(),
                }
                .key(site),
                CommitTarget::Session => continue,
            };
            writes.push(Entry {
                key,
                value,
                timestamp,
            });

            if removes_prim && target == CommitTarget::Document {
                self.drop_document_prim(*prim, timestamp, &mut writes);
            }
        }
        writes
    }

    /// Takes every property and reference layer the document layer states for
    /// a removed prim, answering a deletion for each.
    fn drop_document_prim(&mut self, prim: PrimId, timestamp: u64, writes: &mut Vec<Entry>) {
        let names = self
            .layer(LayerId::Document)
            .get(prim)
            .map_or_default(|opinions| {
                opinions
                    .properties()
                    .map(|(name, _)| name.clone())
                    .collect::<Vec<_>>()
            });
        for name in names {
            self.layer_mut(LayerId::Document).take_property(prim, &name);
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
        target: LayerId,
        prim: PrimId,
        name: &PropName,
    ) -> Option<(Bytes, u64)> {
        let (opinion, _) = self.take_live(prim, |layer, id| layer.take_property(id, name))?;
        let timestamp = self.local_property_time(target, prim, name);
        let (value, stamp) = opinion.value().map_or_else(
            || (Vec::new(), Stamp::new(timestamp, &[])),
            |property| (property.encode(), Stamp::of_property(timestamp, property)),
        );
        let opinions = self.layer_mut(target).entry(prim);
        if target.is_projected() {
            opinions.replace_property(name, opinion, stamp);
        } else {
            opinions.set_property(name, opinion, stamp);
        }
        self.settle_property(prim, name);
        Some((value.into(), timestamp))
    }

    fn commit_parent(&mut self, target: LayerId, prim: PrimId) -> Option<(Bytes, u64)> {
        let (opinion, _) = self.take_live(prim, Layer::take_parent)?;
        let parent = opinion.value().copied();
        let stamp = self.local_parent_stamp(target, prim, parent);
        let opinions = self.layer_mut(target).entry(prim);
        if target.is_projected() {
            opinions.replace_parent(opinion, stamp);
        } else {
            opinions.set_parent(opinion, stamp);
        }
        self.settle_parent(prim);
        Some((ParentAttr::to_wire(parent).into(), stamp.timestamp))
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
