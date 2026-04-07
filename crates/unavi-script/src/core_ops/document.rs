use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use bevy::prelude::{Entity, World};
use bevy_hsd::cache::{
    MaterialHsdChanges, MaterialInner, MaterialState, MeshHsdChanges, MeshInner, MeshState,
    NodeHsdChanges, NodeInner, NodeState, SceneRegistryInner, SyncOp,
};
use bevy_hsd::hydrate::compile::material::{HsdMaterialDespawned, HsdMaterialSpawned};
use bevy_hsd::hydrate::compile::mesh::{HsdMeshDespawned, HsdMeshSpawned};
use bevy_hsd::hydrate::compile::node::{HsdNodeDespawned, HsdNodeSpawned};
use bevy_hsd::hydrate::events::ScriptCommandQueue;

use crate::util::gen_id;

pub fn create_node(
    registry: &SceneRegistryInner,
    doc_entity: Entity,
    cmds: &mut ScriptCommandQueue,
) -> Arc<NodeInner> {
    let id = gen_id();
    let inner = Arc::new(NodeInner {
        entity: Mutex::new(None),
        hsd_changes: Mutex::new(NodeHsdChanges::default()),
        id: id.clone(),
        is_virtual: false,
        state: Mutex::new(NodeState::default()),
        sync: false.into(),
        tree_id: Mutex::new(None),
    });
    registry
        .nodes
        .lock()
        .expect("nodes lock")
        .push(Arc::clone(&inner));
    registry
        .node_map
        .lock()
        .expect("node_map lock")
        .insert(id.clone(), Arc::clone(&inner));
    let id_ = id.clone();
    cmds.push(move |world: &mut World| {
        world.trigger(HsdNodeSpawned {
            doc: doc_entity,
            id: id_,
        });
    });
    if registry.doc_sync.load(Ordering::Relaxed) {
        registry
            .pending_doc_ops
            .lock()
            .expect("pending_doc_ops lock")
            .push(SyncOp::NodeCreated(id));
    }
    inner
}

pub fn create_mesh(
    registry: &SceneRegistryInner,
    doc_entity: Entity,
    cmds: &mut ScriptCommandQueue,
) -> Arc<MeshInner> {
    let id = gen_id();
    let inner = Arc::new(MeshInner {
        entity: Mutex::new(None),
        hsd_changes: Mutex::new(MeshHsdChanges::default()),
        id: id.clone(),
        state: Mutex::new(MeshState::default()),
        sync: false.into(),
    });
    registry
        .meshes
        .lock()
        .expect("meshes lock")
        .insert(id.clone(), Arc::clone(&inner));
    let id_ = id.clone();
    cmds.push(move |world: &mut World| {
        world.trigger(HsdMeshSpawned {
            doc: doc_entity,
            id: id_,
        });
    });
    if registry.doc_sync.load(Ordering::Relaxed) {
        registry
            .pending_doc_ops
            .lock()
            .expect("pending_doc_ops lock")
            .push(SyncOp::MeshCreated(id));
    }
    inner
}

pub fn create_material(
    registry: &SceneRegistryInner,
    doc_entity: Entity,
    cmds: &mut ScriptCommandQueue,
) -> Arc<MaterialInner> {
    let id = gen_id();
    let inner = Arc::new(MaterialInner {
        entity: Mutex::new(None),
        hsd_changes: Mutex::new(MaterialHsdChanges::default()),
        id: id.clone(),
        state: Mutex::new(MaterialState::default()),
        sync: false.into(),
    });
    registry
        .materials
        .lock()
        .expect("materials lock")
        .insert(id.clone(), Arc::clone(&inner));
    let id_ = id.clone();
    cmds.push(move |world: &mut World| {
        world.trigger(HsdMaterialSpawned {
            doc: doc_entity,
            id: id_,
            initial: None,
        });
    });
    if registry.doc_sync.load(Ordering::Relaxed) {
        registry
            .pending_doc_ops
            .lock()
            .expect("pending_doc_ops lock")
            .push(SyncOp::MaterialCreated(id));
    }
    inner
}

pub fn remove_node(
    inner: &NodeInner,
    registry: &SceneRegistryInner,
    doc_entity: Entity,
    cmds: &mut ScriptCommandQueue,
) {
    let id = inner.id.clone();
    cmds.push(move |world: &mut World| {
        world.trigger(HsdNodeDespawned {
            doc: doc_entity,
            id,
        });
    });
    if registry.doc_sync.load(Ordering::Relaxed) {
        let id = inner.id.clone();
        registry
            .pending_doc_ops
            .lock()
            .expect("pending_doc_ops lock")
            .push(SyncOp::NodeRemoved(id));
    }
}

pub fn remove_mesh(
    inner: &MeshInner,
    registry: &SceneRegistryInner,
    doc_entity: Entity,
    cmds: &mut ScriptCommandQueue,
) {
    let id = inner.id.clone();
    cmds.push(move |world: &mut World| {
        world.trigger(HsdMeshDespawned {
            doc: doc_entity,
            id,
        });
    });
    if registry.doc_sync.load(Ordering::Relaxed) {
        let id = inner.id.clone();
        registry
            .pending_doc_ops
            .lock()
            .expect("pending_doc_ops lock")
            .push(SyncOp::MeshRemoved(id));
    }
}

pub fn remove_material(
    inner: &MaterialInner,
    registry: &SceneRegistryInner,
    doc_entity: Entity,
    cmds: &mut ScriptCommandQueue,
) {
    let id = inner.id.clone();
    cmds.push(move |world: &mut World| {
        world.trigger(HsdMaterialDespawned {
            doc: doc_entity,
            id,
        });
    });
    if registry.doc_sync.load(Ordering::Relaxed) {
        let id = inner.id.clone();
        registry
            .pending_doc_ops
            .lock()
            .expect("pending_doc_ops lock")
            .push(SyncOp::MaterialRemoved(id));
    }
}

pub fn set_sync(registry: &SceneRegistryInner, value: bool) {
    registry.doc_sync.store(value, Ordering::Relaxed);
    if value {
        let mut ops = registry
            .pending_doc_ops
            .lock()
            .expect("pending_doc_ops lock");
        let nodes = registry.nodes.lock().expect("nodes lock");
        for n in nodes.iter() {
            ops.push(SyncOp::NodeCreated(n.id.clone()));
        }
        drop(nodes);
        let meshes = registry.meshes.lock().expect("meshes lock");
        for id in meshes.keys() {
            ops.push(SyncOp::MeshCreated(id.clone()));
        }
        drop(meshes);
        let materials = registry.materials.lock().expect("materials lock");
        for id in materials.keys() {
            ops.push(SyncOp::MaterialCreated(id.clone()));
        }
    }
}
