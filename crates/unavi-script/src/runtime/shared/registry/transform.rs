use std::{
    collections::HashMap,
    sync::LazyLock,
};

use bevy::prelude::*;
use bevy_hsd::{
    HsdChild,
    HsdRecordId,
    Prim,
};
use blake3::Hash;
use loro::TreeID;
use parking_lot::RwLock;

pub static NODE_TRANSFORM_REGISTRY: LazyLock<RwLock<HashMap<AbsoluteNodeId, TransformSnapshot>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub static DOC_ROOT_TRANSFORM_REGISTRY: LazyLock<RwLock<HashMap<Hash, GlobalTransform>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct AbsoluteNodeId {
    pub doc:  Hash,
    pub node: TreeID,
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
) {
    if transforms.is_empty() {
        return;
    }

    let mut reg = NODE_TRANSFORM_REGISTRY.write();

    for (id, global, local, doc) in transforms {
        let doc_relative = doc
            .and_then(|c| docs.get(c.0).ok())
            .map_or(*global, |doc_global| {
                GlobalTransform::from(doc_global.affine().inverse() * global.affine())
            });
        reg.insert(
            id.0.clone(),
            TransformSnapshot {
                global: doc_relative,
                local:  *local,
                world:  *global,
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

pub fn snapshot_doc_roots(docs: Query<(&HsdRecordId, &GlobalTransform), With<bevy_hsd::Hsd>>) {
    if docs.is_empty() {
        return;
    }
    let mut reg = DOC_ROOT_TRANSFORM_REGISTRY.write();
    for (record, global) in &docs {
        reg.insert(record.0, *global);
    }
}

pub fn deregister_doc_root(trigger: On<Remove, bevy_hsd::Hsd>, docs: Query<&HsdRecordId>) {
    if let Ok(record) = docs.get(trigger.entity) {
        DOC_ROOT_TRANSFORM_REGISTRY.write().remove(&record.0);
    }
}
