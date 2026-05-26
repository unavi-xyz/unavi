use std::{collections::HashMap, sync::LazyLock};

use bevy::prelude::*;
use bevy_hsd::{HsdChild, HsdRecordId, Prim};
use blake3::Hash;
use loro::TreeID;
use parking_lot::RwLock;

pub static NODE_TRANSFORM_REGISTRY: LazyLock<RwLock<HashMap<AbsoluteNodeId, TransformSnapshot>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

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
    trigger: On<Add, Prim>,
    prims: Query<(&Prim, &HsdChild)>,
    docs: Query<&HsdRecordId>,
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
            doc: doc.0,
            node: prim.0,
        }));
}

pub fn snapshot_transforms(transforms: Query<(&RegisterTransforms, &GlobalTransform, &Transform)>) {
    if transforms.is_empty() {
        return;
    }

    let mut reg = NODE_TRANSFORM_REGISTRY.write();

    for (id, global, local) in transforms {
        reg.insert(
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
    NODE_TRANSFORM_REGISTRY.write().remove(&id.0);
}
