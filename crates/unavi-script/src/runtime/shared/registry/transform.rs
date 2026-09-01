use std::{
    collections::HashMap,
    sync::Arc,
};

use bevy::{
    math::Affine3A,
    prelude::*,
};
use bevy_hsd::{
    HsdChild,
    HsdDocId,
    Prim,
};
use hsd::id::{
    DocId,
    PrimId,
};
use parking_lot::RwLock;

/// This frame's node poses and per-document root transforms, for scripts to
/// read off-thread and for the host to resolve a prim's world transform
/// without walking the ECS.
#[derive(Resource, Clone, Default)]
pub struct TransformSnapshots(Arc<Inner>);

#[derive(Default)]
struct Inner {
    nodes:     RwLock<HashMap<AbsoluteNodeId, TransformSnapshot>>,
    doc_roots: RwLock<HashMap<DocId, GlobalTransform>>,
}

impl TransformSnapshots {
    pub fn snapshot_nodes(
        &self,
        entries: impl Iterator<Item = (AbsoluteNodeId, TransformSnapshot)>,
    ) {
        let mut nodes = self.0.nodes.write();
        for (id, snapshot) in entries {
            nodes.insert(id, snapshot);
        }
    }

    pub fn snapshot_doc_roots(&self, entries: impl Iterator<Item = (DocId, GlobalTransform)>) {
        let mut doc_roots = self.0.doc_roots.write();
        for (doc, transform) in entries {
            doc_roots.insert(doc, transform);
        }
    }

    pub fn remove_node(&self, id: &AbsoluteNodeId) {
        self.0.nodes.write().remove(id);
    }

    pub fn remove_doc_root(&self, doc: &DocId) {
        self.0.doc_roots.write().remove(doc);
    }

    #[must_use]
    pub fn node(&self, id: &AbsoluteNodeId) -> Option<TransformSnapshot> {
        self.0.nodes.read().get(id).cloned()
    }

    #[must_use]
    pub fn doc_root(&self, doc: &DocId) -> Option<GlobalTransform> {
        self.0.doc_roots.read().get(doc).copied()
    }

    /// Back-patches `id`'s local transform, so a script reads back what it
    /// just wrote without waiting for the next snapshot. A no-op if `id` has
    /// no entry yet: the write still lands in the attribute, and the next
    /// snapshot will pick it up.
    pub fn set_local(&self, id: &AbsoluteNodeId, local: Transform) {
        if let Some(snapshot) = self.0.nodes.write().get_mut(id) {
            snapshot.local = local;
        }
    }

    /// `leaf`'s world affine, composed from its own and every ancestor's local
    /// transform (via `parent_of`) and `doc`'s root. One read lock covers the
    /// whole ancestor walk, rather than one per node visited.
    pub fn world_of(
        &self,
        doc: DocId,
        leaf: PrimId,
        mut parent_of: impl FnMut(PrimId) -> anyhow::Result<Option<PrimId>>,
    ) -> anyhow::Result<Affine3A> {
        // Walk the chain before taking the lock, so it's never held across
        // `parent_of`'s own locking (the ECS parent walk).
        let mut chain = vec![leaf];
        let mut cur = leaf;
        while let Some(id) = parent_of(cur)? {
            chain.push(id);
            cur = id;
        }

        let nodes = self.0.nodes.read();
        let mut local = Affine3A::IDENTITY;
        for id in chain {
            let t = nodes
                .get(&AbsoluteNodeId { doc, node: id })
                .map(|s| s.local)
                .unwrap_or_default();
            local = t.compute_affine() * local;
        }
        drop(nodes);
        let root = self
            .doc_root(&doc)
            .map_or(Affine3A::IDENTITY, |g| g.affine());
        Ok(root * local)
    }
}

/// Keyed by document id rather than namespace: a prefab instance has an id
/// from birth but no namespace at all.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct AbsoluteNodeId {
    pub doc:  DocId,
    pub node: PrimId,
}

#[derive(Clone, Default)]
pub struct TransformSnapshot {
    pub global: GlobalTransform,
    pub local:  Transform,
    pub world:  GlobalTransform,
}

#[derive(Component)]
#[require(Transform)]
pub struct RegisterTransforms(pub AbsoluteNodeId);

pub fn register_nodes(
    trigger: On<Add, Prim>,
    prims: Query<(&Prim, &HsdChild)>,
    docs: Query<&HsdDocId>,
    mut commands: Commands,
) {
    let Ok((prim, doc)) = prims.get(trigger.entity) else {
        error!("unable to register prim: prim not found");
        return;
    };
    let Ok(doc) = docs.get(doc.0) else {
        error!("unable to register prim: document not found");
        return;
    };
    commands
        .entity(trigger.entity)
        .insert(RegisterTransforms(AbsoluteNodeId {
            doc:  doc.0,
            node: prim.0,
        }));
}

pub fn snapshot_transforms(
    transforms: Query<(
        &RegisterTransforms,
        &GlobalTransform,
        &Transform,
        Option<&HsdChild>,
    )>,
    docs: Query<&GlobalTransform>,
    registry: Res<TransformSnapshots>,
) {
    if transforms.is_empty() {
        return;
    }

    registry.snapshot_nodes(transforms.iter().map(|(id, global, local, doc)| {
        let doc_relative = doc
            .and_then(|c| docs.get(c.0).ok())
            .map_or(*global, |doc_global| {
                GlobalTransform::from(doc_global.affine().inverse() * global.affine())
            });
        (
            id.0,
            TransformSnapshot {
                global: doc_relative,
                local:  *local,
                world:  *global,
            },
        )
    }));
}

pub fn deregister_transforms(
    trigger: On<Remove, RegisterTransforms>,
    ids: Query<&RegisterTransforms>,
    registry: Res<TransformSnapshots>,
) {
    let id = ids.get(trigger.entity).expect("id");
    registry.remove_node(&id.0);
}

pub fn snapshot_doc_roots(
    docs: Query<(&HsdDocId, &GlobalTransform), With<bevy_hsd::Hsd>>,
    registry: Res<TransformSnapshots>,
) {
    if docs.is_empty() {
        return;
    }
    registry.snapshot_doc_roots(docs.iter().map(|(record, global)| (record.0, *global)));
}

pub fn deregister_doc_root(
    trigger: On<Remove, bevy_hsd::Hsd>,
    docs: Query<&HsdDocId>,
    registry: Res<TransformSnapshots>,
) {
    if let Ok(record) = docs.get(trigger.entity) {
        registry.remove_doc_root(&record.0);
    }
}
