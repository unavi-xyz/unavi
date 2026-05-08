use std::sync::LazyLock;

use bevy::prelude::*;
use bevy_hsd::{HsdChild, HsdRecordId, NodeId};
use blake3::Hash;
use loro::TreeID;

pub static NODE_TRANSFORM_REGISTRY: LazyLock<scc::HashMap<AbsoluteNodeId, TransformSnapshot>> =
    LazyLock::new(scc::HashMap::default);

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct AbsoluteNodeId {
    pub doc: Hash,
    pub node: TreeID,
}

#[derive(Clone, Default)]
pub struct TransformSnapshot {
    pub global: GlobalTransform,
    pub local: Transform,
}

#[derive(Component)]
#[require(Transform)]
pub struct RegisterTransforms(pub AbsoluteNodeId);

pub fn register_nodes(
    trigger: On<Add, NodeId>,
    nodes: Query<(&NodeId, &HsdChild)>,
    docs: Query<&HsdRecordId>,
    mut commands: Commands,
) {
    let Ok((node, doc)) = nodes.get(trigger.entity) else {
        error!("unable to register node: node not found");
        return;
    };
    let Ok(doc) = docs.get(doc.0) else {
        error!("unable to register node: document not found");
        return;
    };
    commands
        .entity(trigger.entity)
        .insert(RegisterTransforms(AbsoluteNodeId {
            doc: doc.0,
            node: node.0,
        }));
}

pub fn snapshot_transforms(transforms: Query<(&RegisterTransforms, &GlobalTransform, &Transform)>) {
    for (id, global, local) in transforms {
        NODE_TRANSFORM_REGISTRY.upsert_sync(
            id.0.clone(),
            TransformSnapshot {
                global: *global,
                local: *local,
            },
        );
    }
}

pub fn deregister_transforms(
    trigger: On<Remove, RegisterTransforms>,
    ids: Query<&RegisterTransforms>,
) {
    let id = ids.get(trigger.entity).expect("id");
    NODE_TRANSFORM_REGISTRY.remove_sync(&id.0);
}
