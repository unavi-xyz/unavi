use std::sync::{Arc, Mutex, atomic::Ordering};

use bevy_hsd::cache::{
    MaterialDirty, MaterialHsdChanges, MaterialInner, MaterialState, MeshDirty, MeshHsdChanges,
    MeshInner, MeshState, NodeDirty, NodeHsdChanges, NodeInner, NodeState, SyncOp,
};
use bevy_hsd::hydrate::events::ScriptQueuedEvent;
use rand::{Rng, distr::Alphanumeric};
use smol_str::SmolStr;
use wasmtime::component::Resource;

use super::bindings::wired::scene::types::{Document, Material, Mesh};
use crate::api::wired::scene::{
    WiredSceneRt, material::HostMaterial, mesh::HostMesh, node::HostNode,
};

#[derive(Clone)]
pub struct HostDocument {
    pub id: blake3::Hash,
}

pub fn gen_id() -> SmolStr {
    /// Max byte length for an inline [`SmolStr`].
    const MAX_INLINE: usize = 23;

    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(MAX_INLINE)
        .map(char::from)
        .collect::<SmolStr>()
}

impl super::bindings::wired::scene::types::HostDocument for WiredSceneRt {
    async fn clone(
        &mut self,
        self_: wasmtime::component::Resource<HostDocument>,
    ) -> wasmtime::Result<wasmtime::component::Resource<HostDocument>> {
        let inner = self.table.get(&self_)?.clone();
        let doc = self.table.push(inner)?;
        Ok(doc)
    }

    async fn create_material(
        &mut self,
        _self_: Resource<Document>,
    ) -> wasmtime::Result<Resource<Material>> {
        let id = gen_id();
        let inner = Arc::new(MaterialInner {
            dirty: Mutex::new(MaterialDirty::default()),
            entity: Mutex::new(None),
            hsd_changes: Mutex::new(MaterialHsdChanges::default()),
            id: id.clone(),
            state: Mutex::new(MaterialState::default()),
            sync: false.into(),
        });
        self.registry
            .materials
            .lock()
            .expect("materials lock")
            .insert(id.clone(), Arc::clone(&inner));
        self.push_script_event(ScriptQueuedEvent::MaterialSpawned { id: id.clone() });
        if self.registry.doc_sync.load(Ordering::Relaxed) {
            self.registry
                .pending_doc_ops
                .lock()
                .expect("pending_doc_ops lock")
                .push(SyncOp::MaterialCreated(id));
        }
        Ok(self.table.push(HostMaterial { inner })?)
    }

    async fn create_mesh(
        &mut self,
        _self_: Resource<Document>,
    ) -> wasmtime::Result<Resource<Mesh>> {
        let id = gen_id();
        let inner = Arc::new(MeshInner {
            dirty: Mutex::new(MeshDirty::default()),
            entity: Mutex::new(None),
            hsd_changes: Mutex::new(MeshHsdChanges::default()),
            id: id.clone(),
            state: Mutex::new(MeshState::default()),
            sync: false.into(),
        });
        self.registry
            .meshes
            .lock()
            .expect("meshes lock")
            .insert(id.clone(), Arc::clone(&inner));
        self.push_script_event(ScriptQueuedEvent::MeshSpawned { id: id.clone() });
        if self.registry.doc_sync.load(Ordering::Relaxed) {
            self.registry
                .pending_doc_ops
                .lock()
                .expect("pending_doc_ops lock")
                .push(SyncOp::MeshCreated(id));
        }
        Ok(self.table.push(HostMesh { inner })?)
    }

    async fn create_node(
        &mut self,
        _self_: Resource<Document>,
    ) -> wasmtime::Result<Resource<HostNode>> {
        let id = gen_id();
        let inner = Arc::new(NodeInner {
            dirty: Mutex::new(NodeDirty::default()),
            entity: Mutex::new(None),
            hsd_changes: Mutex::new(NodeHsdChanges::default()),
            id: id.clone(),
            is_virtual: false,
            state: Mutex::new(NodeState::default()),
            sync: false.into(),
            tree_id: Mutex::new(None),
        });
        self.registry
            .nodes
            .lock()
            .expect("nodes lock")
            .push(Arc::clone(&inner));
        self.registry
            .node_map
            .lock()
            .expect("node_map lock")
            .insert(id.clone(), Arc::clone(&inner));
        self.push_script_event(ScriptQueuedEvent::NodeSpawned { id: id.clone() });
        if self.registry.doc_sync.load(Ordering::Relaxed) {
            self.registry
                .pending_doc_ops
                .lock()
                .expect("pending_doc_ops lock")
                .push(SyncOp::NodeCreated(id));
        }
        Ok(self.table.push(HostNode { inner })?)
    }

    async fn roots(
        &mut self,
        _self_: Resource<Document>,
    ) -> wasmtime::Result<Vec<Resource<HostNode>>> {
        let nodes: Vec<Arc<NodeInner>> = {
            let all = self.registry.nodes.lock().expect("nodes lock");
            all.iter()
                .filter(|n| {
                    n.state
                        .lock()
                        .expect("node state lock")
                        .parent
                        .as_ref()
                        .is_none_or(|w| w.upgrade().is_none())
                })
                .cloned()
                .collect()
        };
        let mut out = Vec::with_capacity(nodes.len());
        for inner in nodes {
            out.push(self.table.push(HostNode { inner })?);
        }
        Ok(out)
    }

    async fn nodes(
        &mut self,
        _self_: Resource<Document>,
    ) -> wasmtime::Result<Vec<Resource<HostNode>>> {
        let nodes: Vec<Arc<NodeInner>> = self.registry.nodes.lock().expect("nodes lock").clone();
        let mut out = Vec::with_capacity(nodes.len());
        for inner in nodes {
            out.push(self.table.push(HostNode { inner })?);
        }
        Ok(out)
    }

    async fn meshes(
        &mut self,
        _self_: Resource<Document>,
    ) -> wasmtime::Result<Vec<Resource<HostMesh>>> {
        let inners: Vec<Arc<MeshInner>> = self
            .registry
            .meshes
            .lock()
            .expect("meshes lock")
            .values()
            .cloned()
            .collect();
        let mut out = Vec::with_capacity(inners.len());
        for inner in inners {
            out.push(self.table.push(HostMesh { inner })?);
        }
        Ok(out)
    }

    async fn materials(
        &mut self,
        _self_: Resource<Document>,
    ) -> wasmtime::Result<Vec<Resource<HostMaterial>>> {
        let inners: Vec<Arc<MaterialInner>> = self
            .registry
            .materials
            .lock()
            .expect("materials lock")
            .values()
            .cloned()
            .collect();
        let mut out = Vec::with_capacity(inners.len());
        for inner in inners {
            out.push(self.table.push(HostMaterial { inner })?);
        }
        Ok(out)
    }

    async fn remove_node(
        &mut self,
        _self_: Resource<Document>,
        value: Resource<HostNode>,
    ) -> wasmtime::Result<()> {
        let inner = Arc::clone(&self.table.get(&value)?.inner);
        let id = inner.id.clone();
        self.push_script_event(ScriptQueuedEvent::NodeDespawned { id: id.clone() });
        if self.registry.doc_sync.load(Ordering::Relaxed) {
            self.registry
                .pending_doc_ops
                .lock()
                .expect("pending_doc_ops lock")
                .push(SyncOp::NodeRemoved(id));
        }
        Ok(())
    }

    async fn remove_mesh(
        &mut self,
        _self_: Resource<Document>,
        value: Resource<HostMesh>,
    ) -> wasmtime::Result<()> {
        let inner = Arc::clone(&self.table.get(&value)?.inner);
        let id = inner.id.clone();
        self.push_script_event(ScriptQueuedEvent::MeshDespawned { id: id.clone() });
        if self.registry.doc_sync.load(Ordering::Relaxed) {
            self.registry
                .pending_doc_ops
                .lock()
                .expect("pending_doc_ops lock")
                .push(SyncOp::MeshRemoved(id));
        }
        Ok(())
    }

    async fn remove_material(
        &mut self,
        _self_: Resource<Document>,
        value: Resource<HostMaterial>,
    ) -> wasmtime::Result<()> {
        let inner = Arc::clone(&self.table.get(&value)?.inner);
        let id = inner.id.clone();
        self.push_script_event(ScriptQueuedEvent::MaterialDespawned { id: id.clone() });
        if self.registry.doc_sync.load(Ordering::Relaxed) {
            self.registry
                .pending_doc_ops
                .lock()
                .expect("pending_doc_ops lock")
                .push(SyncOp::MaterialRemoved(id));
        }
        Ok(())
    }

    async fn sync(&mut self, _self_: Resource<Document>) -> wasmtime::Result<bool> {
        Ok(self.registry.doc_sync.load(Ordering::Relaxed))
    }

    async fn set_sync(&mut self, _self_: Resource<Document>, value: bool) -> wasmtime::Result<()> {
        if value {
            self.check_hsd_write()?;
        }
        self.registry.doc_sync.store(value, Ordering::Relaxed);
        if value {
            let mut ops = self
                .registry
                .pending_doc_ops
                .lock()
                .expect("pending_doc_ops lock");
            let nodes = self.registry.nodes.lock().expect("nodes lock");
            for n in nodes.iter() {
                ops.push(SyncOp::NodeCreated(n.id.clone()));
            }
            drop(nodes);
            let meshes = self.registry.meshes.lock().expect("meshes lock");
            for id in meshes.keys() {
                ops.push(SyncOp::MeshCreated(id.clone()));
            }
            drop(meshes);
            let materials = self.registry.materials.lock().expect("materials lock");
            for id in materials.keys() {
                ops.push(SyncOp::MaterialCreated(id.clone()));
            }
        }
        Ok(())
    }

    async fn id(&mut self, self_: Resource<Document>) -> wasmtime::Result<Vec<u8>> {
        Ok(self.table.get(&self_)?.id.as_bytes().to_vec())
    }

    async fn drop(&mut self, rep: Resource<Document>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}
