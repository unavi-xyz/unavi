//! This frame's poses, for scripts to read off the main thread.

use std::{
    collections::{
        HashMap,
        HashSet,
    },
    mem,
    sync::Arc,
};

use bevy::{
    math::Affine3A,
    prelude::*,
};
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
use parking_lot::{
    Mutex,
    RwLock,
};

/// This frame's node poses and per-document root transforms, for scripts to
/// read off-thread and for the host to resolve a prim's world transform
/// without walking the ECS.
#[derive(Resource, Clone, Default)]
pub struct TransformSnapshots(Arc<Inner>);

#[derive(Default)]
struct Inner {
    nodes:     RwLock<HashMap<AbsoluteNodeId, TransformSnapshot>>,
    doc_roots: RwLock<HashMap<DocId, GlobalTransform>>,
    patched:   Mutex<Patched>,
}

/// Nodes [`TransformSnapshots::set_local`] back-patched, by how many frames
/// have passed since. A write that never reaches the ECS `Transform` leaves no
/// change to refresh the node, so [`refresh_patched`] refreshes it regardless
/// a whole frame after the patch, by when a write that lands has landed.
#[derive(Default)]
struct Patched {
    pending: HashSet<AbsoluteNodeId>,
    aged:    HashSet<AbsoluteNodeId>,
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
            self.0.patched.lock().pending.insert(*id);
        }
    }

    /// Ages this frame's patches, answering those due a refresh.
    fn take_stale(&self) -> HashSet<AbsoluteNodeId> {
        let mut patched = self.0.patched.lock();
        let pending = mem::take(&mut patched.pending);
        mem::replace(&mut patched.aged, pending)
    }

    /// `leaf`'s world affine, composed from its own and every ancestor's local
    /// transform (via `parent_of`) and `doc`'s root. One read lock covers the
    /// whole ancestor walk, rather than one per node visited.
    pub fn world_of(
        &self,
        doc: DocId,
        leaf: PrimId,
        mut parent_of: impl FnMut(PrimId) -> Option<PrimId>,
    ) -> Affine3A {
        // Walk the chain before taking the lock, so it's never held across
        // `parent_of`'s own locking (the ECS parent walk).
        let mut chain = vec![leaf];
        let mut cur = leaf;
        while let Some(id) = parent_of(cur) {
            chain.push(id);
            cur = id;
        }

        let nodes = self.0.nodes.read();
        let mut local = Affine3A::IDENTITY;
        for id in chain {
            let t = nodes
                .get(&AbsoluteNodeId { doc, node: id })
                .map_or_default(|s| s.local);
            local = t.compute_affine() * local;
        }
        drop(nodes);
        let root = self
            .doc_root(&doc)
            .map_or(Affine3A::IDENTITY, |g| g.affine());
        root * local
    }
}

/// Keyed by document id rather than namespace: a reference site has an id
/// from birth but no namespace at all.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct AbsoluteNodeId {
    pub doc:  DocId,
    pub node: PrimId,
}

#[derive(Clone, Default)]
pub struct TransformSnapshot {
    pub local: Transform,
    pub world: GlobalTransform,
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

/// A node whose own transform and propagated global are both unchanged keeps
/// its snapshot. Propagation marks a moved subtree's `GlobalTransform`, so the
/// filter never misses one.
pub fn snapshot_transforms(
    changed: Query<
        (&RegisterTransforms, &GlobalTransform, &Transform),
        Or<(Changed<GlobalTransform>, Changed<Transform>)>,
    >,
    registry: Res<TransformSnapshots>,
) {
    if !changed.is_empty() {
        registry.snapshot_nodes(changed.iter().map(snapshot));
    }
}

/// Rewrites back-patched nodes from the ECS once their patch is a frame old,
/// whether or not the write changed anything. Runs once a frame, so a patch
/// outlives every snapshot of the frame it was made in.
pub fn refresh_patched(
    nodes: Query<(&RegisterTransforms, &GlobalTransform, &Transform)>,
    registry: Res<TransformSnapshots>,
) {
    let stale = registry.take_stale();
    if stale.is_empty() {
        return;
    }
    registry.snapshot_nodes(
        nodes
            .iter()
            .filter(|(id, ..)| stale.contains(&id.0))
            .map(snapshot),
    );
}

const fn snapshot(
    (id, global, local): (&RegisterTransforms, &GlobalTransform, &Transform),
) -> (AbsoluteNodeId, TransformSnapshot) {
    (
        id.0,
        TransformSnapshot {
            local: *local,
            world: *global,
        },
    )
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
        app.init_resource::<TransformSnapshots>().add_systems(
            Update,
            (refresh_patched, snapshot_transforms, snapshot_doc_roots).chain(),
        );
        app
    }

    fn local(app: &App) -> Option<Vec3> {
        app.world()
            .resource::<TransformSnapshots>()
            .node(&ID)
            .map(|snapshot| snapshot.local.translation)
    }

    fn spawn_node(app: &mut App) -> Entity {
        let prim = app
            .world_mut()
            .spawn((
                RegisterTransforms(ID),
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();
        app.update();
        prim
    }

    #[test]
    fn a_back_patch_reads_back_for_a_frame_and_then_reverts() {
        let mut app = app();
        spawn_node(&mut app);
        let patch = Vec3::new(1.0, 2.0, 3.0);
        app.world()
            .resource::<TransformSnapshots>()
            .set_local(&ID, Transform::from_translation(patch));

        app.update();
        assert_eq!(local(&app), Some(patch), "the patch was dropped early");

        app.update();
        assert_eq!(
            local(&app),
            Some(Vec3::ZERO),
            "a patch the ECS never took is still read back"
        );
    }

    #[test]
    fn a_changed_node_is_rewritten() {
        let mut app = app();
        let prim = spawn_node(&mut app);
        let moved = Vec3::new(4.0, 5.0, 6.0);
        app.world_mut()
            .get_mut::<Transform>(prim)
            .expect("transform")
            .translation = moved;

        app.update();
        assert_eq!(local(&app), Some(moved));
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
