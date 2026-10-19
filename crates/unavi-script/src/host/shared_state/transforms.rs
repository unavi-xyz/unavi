//! This frame's poses, for scripts to read off the main thread.

use std::{
    collections::HashMap,
    sync::Arc,
};

use bevy::prelude::*;
use bevy_hsd::{
    document::{
        Hsd,
        HsdDocId,
        Unplaced,
    },
    prim::{
        Prim,
        PrimOf,
    },
};
use hsd::id::{
    DocId,
    PrimId,
};
use parking_lot::RwLock;

/// Where every prim and document root is this frame, as the engine placed
/// them, for scripts to read off the main thread.
///
/// Observed poses only, never the transforms scripts wrote: a write the
/// session layer refuses, a stronger layer's opinion, or physics all leave the
/// pose somewhere other than where it was written. What was written is the
/// document's to answer.
#[derive(Resource, Clone, Default)]
pub struct TransformSnapshots(Arc<Inner>);

#[derive(Default)]
struct Inner {
    nodes:     RwLock<HashMap<AbsoluteNodeId, GlobalTransform>>,
    doc_roots: RwLock<HashMap<DocId, GlobalTransform>>,
}

impl TransformSnapshots {
    pub fn snapshot_nodes(&self, entries: impl Iterator<Item = (AbsoluteNodeId, GlobalTransform)>) {
        let mut nodes = self.0.nodes.write();
        for (id, world) in entries {
            nodes.insert(id, world);
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

    /// `None` for a prim no snapshot has seen yet, such as one created this
    /// tick.
    #[must_use]
    pub fn node(&self, id: &AbsoluteNodeId) -> Option<GlobalTransform> {
        self.0.nodes.read().get(id).copied()
    }

    #[must_use]
    pub fn doc_root(&self, doc: &DocId) -> Option<GlobalTransform> {
        self.0.doc_roots.read().get(doc).copied()
    }
}

/// Keyed by document id rather than namespace: a reference site has an id
/// from birth but no namespace at all.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct AbsoluteNodeId {
    pub doc:  DocId,
    pub node: PrimId,
}

#[derive(Component)]
#[require(Transform)]
pub struct RegisterTransforms(pub AbsoluteNodeId);

pub fn register_nodes(
    trigger: On<Add, Prim>,
    prims: Query<(&Prim, &PrimOf)>,
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

/// Both instances run after a transform propagation, which marks every moved
/// subtree's `GlobalTransform`, so the filter never misses a node.
pub fn snapshot_transforms(
    changed: Query<(&RegisterTransforms, &GlobalTransform), Changed<GlobalTransform>>,
    registry: Res<TransformSnapshots>,
) {
    if !changed.is_empty() {
        registry.snapshot_nodes(changed.iter().map(|(id, world)| (id.0, *world)));
    }
}

pub fn deregister_transforms(
    trigger: On<Remove, RegisterTransforms>,
    ids: Query<&RegisterTransforms>,
    registry: Res<TransformSnapshots>,
) {
    if let Ok(id) = ids.get(trigger.entity) {
        registry.remove_node(&id.0);
    }
}

/// A document's root is recorded when it moves, and when it gains its id or
/// leaves [`Unplaced`] at a transform that settled long before.
pub fn snapshot_doc_roots(
    changed: Query<
        (&HsdDocId, &GlobalTransform),
        (
            With<Hsd>,
            Without<Unplaced>,
            Or<(Changed<GlobalTransform>, Changed<HsdDocId>, Added<Hsd>)>,
        ),
    >,
    placed_docs: Query<(&HsdDocId, &GlobalTransform), (With<Hsd>, Without<Unplaced>)>,
    mut placed: RemovedComponents<Unplaced>,
    registry: Res<TransformSnapshots>,
) {
    let placed = placed
        .read()
        .filter_map(|entity| placed_docs.get(entity).ok())
        .collect::<Vec<_>>();
    if changed.is_empty() && placed.is_empty() {
        return;
    }
    registry.snapshot_doc_roots(
        changed
            .iter()
            .chain(placed)
            .map(|(record, global)| (record.0, *global)),
    );
}

pub fn deregister_doc_root(
    trigger: On<Remove, Hsd>,
    docs: Query<&HsdDocId>,
    registry: Res<TransformSnapshots>,
) {
    if let Ok(record) = docs.get(trigger.entity) {
        registry.remove_doc_root(&record.0);
    }
}

#[cfg(test)]
mod tests {
    use hsd::state::HsdState;

    use super::*;

    const ID: AbsoluteNodeId = AbsoluteNodeId {
        doc:  DocId([0; 32]),
        node: PrimId([0; 16]),
    };

    fn app() -> App {
        let mut app = App::new();
        app.init_resource::<TransformSnapshots>()
            .add_systems(Update, (snapshot_transforms, snapshot_doc_roots));
        app
    }

    #[test]
    fn a_moved_node_is_rewritten() {
        let mut app = app();
        let prim = app
            .world_mut()
            .spawn((RegisterTransforms(ID), GlobalTransform::default()))
            .id();
        app.update();

        let moved = Vec3::new(4.0, 5.0, 6.0);
        app.world_mut()
            .entity_mut(prim)
            .insert(GlobalTransform::from_translation(moved));
        app.update();
        assert_eq!(
            app.world()
                .resource::<TransformSnapshots>()
                .node(&ID)
                .map(|world| world.translation()),
            Some(moved)
        );
    }

    fn root(app: &App) -> Option<Vec3> {
        app.world()
            .resource::<TransformSnapshots>()
            .doc_root(&ID.doc)
            .map(|root| root.translation())
    }

    #[test]
    fn a_document_arriving_at_a_settled_transform_records_its_root() {
        let mut app = app();
        let at = Vec3::new(5000.0, 0.0, 0.0);
        let doc = app
            .world_mut()
            .spawn(GlobalTransform::from_translation(at))
            .id();
        app.update();

        app.world_mut()
            .entity_mut(doc)
            .insert((Hsd::new(HsdState::new()), HsdDocId(ID.doc)));
        app.update();
        assert_eq!(root(&app), Some(at));
    }

    #[test]
    fn a_document_leaving_unplaced_records_its_root() {
        let mut app = app();
        let at = Vec3::new(0.0, 0.0, 5000.0);
        let doc = app
            .world_mut()
            .spawn((
                Hsd::new(HsdState::new()),
                HsdDocId(ID.doc),
                Unplaced,
                GlobalTransform::from_translation(at),
            ))
            .id();
        app.update();
        assert_eq!(root(&app), None);

        app.world_mut().entity_mut(doc).remove::<Unplaced>();
        app.update();
        assert_eq!(root(&app), Some(at));
    }
}
