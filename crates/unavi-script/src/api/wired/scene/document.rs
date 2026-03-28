use std::sync::{Arc, Mutex, atomic::Ordering};

use bevy_hsd::cache::{
    MaterialDirty, MaterialHsdChanges, MaterialInner, MaterialState, MeshDirty, MeshHsdChanges,
    MeshInner, MeshState, NodeDirty, NodeHsdChanges, NodeInner, NodeState, SyncOp,
};
use bevy_hsd::hydrate::events::ScriptQueuedEvent;
use bevy_hsd::{cache::SceneRegistryInner, hydrate::events::ScriptQueuedEvent as Ev};
use rand::{Rng, distr::Alphanumeric};
use smol_str::SmolStr;
use wasmtime::component::Resource;

use super::bindings::wired::scene::types::{Document, Material, Mesh};
use crate::api::wired::scene::{
    WiredSceneRt, material::HostMaterial, mesh::HostMesh, node::HostNode,
};

pub struct HostDocument {
    pub id: blake3::Hash,
    pub registry: Arc<SceneRegistryInner>,
    pub events: Arc<Mutex<Vec<ScriptQueuedEvent>>>,
    pub can_read: bool,
    pub can_write: bool,
}

impl Clone for HostDocument {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            registry: Arc::clone(&self.registry),
            events: Arc::clone(&self.events),
            can_read: self.can_read,
            can_write: self.can_write,
        }
    }
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

/// Push a queued event into a doc's event queue.
fn push_ev(events: &Arc<Mutex<Vec<Ev>>>, ev: Ev) {
    events.lock().expect("events lock").push(ev);
}

impl super::bindings::wired::scene::types::HostDocument for WiredSceneRt {
    async fn clone(
        &mut self,
        self_: Resource<HostDocument>,
    ) -> wasmtime::Result<Resource<HostDocument>> {
        let inner = self.table.get(&self_)?.clone();
        Ok(self.table.push(inner)?)
    }

    async fn create_material(
        &mut self,
        self_: Resource<Document>,
    ) -> wasmtime::Result<Resource<Material>> {
        let (registry, events, can_write) = {
            let d = self.table.get(&self_)?;
            (Arc::clone(&d.registry), Arc::clone(&d.events), d.can_write)
        };
        if !can_write {
            return Err(anyhow::anyhow!("hsd write permission required"));
        }
        let id = gen_id();
        let inner = Arc::new(MaterialInner {
            dirty: std::sync::Mutex::new(MaterialDirty::default()),
            entity: std::sync::Mutex::new(None),
            hsd_changes: std::sync::Mutex::new(MaterialHsdChanges::default()),
            id: id.clone(),
            state: std::sync::Mutex::new(MaterialState::default()),
            sync: false.into(),
        });
        registry
            .materials
            .lock()
            .expect("materials lock")
            .insert(id.clone(), Arc::clone(&inner));
        push_ev(&events, Ev::MaterialSpawned { id: id.clone() });
        if registry.doc_sync.load(Ordering::Relaxed) {
            registry
                .pending_doc_ops
                .lock()
                .expect("pending_doc_ops lock")
                .push(SyncOp::MaterialCreated(id));
        }
        Ok(self.table.push(HostMaterial {
            inner,
            can_read: true,
            can_write: true,
        })?)
    }

    async fn create_mesh(&mut self, self_: Resource<Document>) -> wasmtime::Result<Resource<Mesh>> {
        let (registry, events, can_write) = {
            let d = self.table.get(&self_)?;
            (Arc::clone(&d.registry), Arc::clone(&d.events), d.can_write)
        };
        if !can_write {
            return Err(anyhow::anyhow!("hsd write permission required"));
        }
        let id = gen_id();
        let inner = Arc::new(MeshInner {
            dirty: std::sync::Mutex::new(MeshDirty::default()),
            entity: std::sync::Mutex::new(None),
            hsd_changes: std::sync::Mutex::new(MeshHsdChanges::default()),
            id: id.clone(),
            state: std::sync::Mutex::new(MeshState::default()),
            sync: false.into(),
        });
        registry
            .meshes
            .lock()
            .expect("meshes lock")
            .insert(id.clone(), Arc::clone(&inner));
        push_ev(&events, Ev::MeshSpawned { id: id.clone() });
        if registry.doc_sync.load(Ordering::Relaxed) {
            registry
                .pending_doc_ops
                .lock()
                .expect("pending_doc_ops lock")
                .push(SyncOp::MeshCreated(id));
        }
        Ok(self.table.push(HostMesh {
            inner,
            can_read: true,
            can_write: true,
        })?)
    }

    async fn create_node(
        &mut self,
        self_: Resource<Document>,
    ) -> wasmtime::Result<Resource<HostNode>> {
        let (registry, events, can_write) = {
            let d = self.table.get(&self_)?;
            (Arc::clone(&d.registry), Arc::clone(&d.events), d.can_write)
        };
        if !can_write {
            return Err(anyhow::anyhow!("hsd write permission required"));
        }
        let id = gen_id();
        let inner = Arc::new(NodeInner {
            dirty: std::sync::Mutex::new(NodeDirty::default()),
            entity: std::sync::Mutex::new(None),
            hsd_changes: std::sync::Mutex::new(NodeHsdChanges::default()),
            id: id.clone(),
            is_virtual: false,
            state: std::sync::Mutex::new(NodeState::default()),
            sync: false.into(),
            tree_id: std::sync::Mutex::new(None),
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
        push_ev(&events, Ev::NodeSpawned { id: id.clone() });
        if registry.doc_sync.load(Ordering::Relaxed) {
            registry
                .pending_doc_ops
                .lock()
                .expect("pending_doc_ops lock")
                .push(SyncOp::NodeCreated(id));
        }
        Ok(self.table.push(HostNode {
            inner,
            can_read: true,
            can_write: true,
        })?)
    }

    async fn roots(
        &mut self,
        self_: Resource<Document>,
    ) -> wasmtime::Result<Vec<Resource<HostNode>>> {
        let (registry, can_read, can_write) = {
            let d = self.table.get(&self_)?;
            (Arc::clone(&d.registry), d.can_read, d.can_write)
        };
        if !can_read {
            return Err(anyhow::anyhow!("hsd read permission required"));
        }
        let nodes: Vec<Arc<NodeInner>> = {
            let all = registry.nodes.lock().expect("nodes lock");
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
            out.push(self.table.push(HostNode {
                inner,
                can_read,
                can_write,
            })?);
        }
        Ok(out)
    }

    async fn nodes(
        &mut self,
        self_: Resource<Document>,
    ) -> wasmtime::Result<Vec<Resource<HostNode>>> {
        let (registry, can_read, can_write) = {
            let d = self.table.get(&self_)?;
            (Arc::clone(&d.registry), d.can_read, d.can_write)
        };
        if !can_read {
            return Err(anyhow::anyhow!("hsd read permission required"));
        }
        let nodes: Vec<Arc<NodeInner>> = registry.nodes.lock().expect("nodes lock").clone();
        let mut out = Vec::with_capacity(nodes.len());
        for inner in nodes {
            out.push(self.table.push(HostNode {
                inner,
                can_read,
                can_write,
            })?);
        }
        Ok(out)
    }

    async fn meshes(
        &mut self,
        self_: Resource<Document>,
    ) -> wasmtime::Result<Vec<Resource<HostMesh>>> {
        let (registry, can_read, can_write) = {
            let d = self.table.get(&self_)?;
            (Arc::clone(&d.registry), d.can_read, d.can_write)
        };
        if !can_read {
            return Err(anyhow::anyhow!("hsd read permission required"));
        }
        let inners: Vec<Arc<MeshInner>> = registry
            .meshes
            .lock()
            .expect("meshes lock")
            .values()
            .cloned()
            .collect();
        let mut out = Vec::with_capacity(inners.len());
        for inner in inners {
            out.push(self.table.push(HostMesh {
                inner,
                can_read,
                can_write,
            })?);
        }
        Ok(out)
    }

    async fn materials(
        &mut self,
        self_: Resource<Document>,
    ) -> wasmtime::Result<Vec<Resource<HostMaterial>>> {
        let (registry, can_read, can_write) = {
            let d = self.table.get(&self_)?;
            (Arc::clone(&d.registry), d.can_read, d.can_write)
        };
        if !can_read {
            return Err(anyhow::anyhow!("hsd read permission required"));
        }
        let inners: Vec<Arc<MaterialInner>> = registry
            .materials
            .lock()
            .expect("materials lock")
            .values()
            .cloned()
            .collect();
        let mut out = Vec::with_capacity(inners.len());
        for inner in inners {
            out.push(self.table.push(HostMaterial {
                inner,
                can_read,
                can_write,
            })?);
        }
        Ok(out)
    }

    async fn remove_node(
        &mut self,
        self_: Resource<Document>,
        value: Resource<HostNode>,
    ) -> wasmtime::Result<()> {
        let (events, can_write) = {
            let d = self.table.get(&self_)?;
            (Arc::clone(&d.events), d.can_write)
        };
        if !can_write {
            return Err(anyhow::anyhow!("hsd write permission required"));
        }
        let inner = Arc::clone(&self.table.get(&value)?.inner);
        let id = inner.id.clone();
        push_ev(&events, Ev::NodeDespawned { id: id.clone() });
        if self
            .table
            .get(&self_)?
            .registry
            .doc_sync
            .load(Ordering::Relaxed)
        {
            self.table
                .get(&self_)?
                .registry
                .pending_doc_ops
                .lock()
                .expect("pending_doc_ops lock")
                .push(SyncOp::NodeRemoved(id));
        }
        Ok(())
    }

    async fn remove_mesh(
        &mut self,
        self_: Resource<Document>,
        value: Resource<HostMesh>,
    ) -> wasmtime::Result<()> {
        let (events, can_write) = {
            let d = self.table.get(&self_)?;
            (Arc::clone(&d.events), d.can_write)
        };
        if !can_write {
            return Err(anyhow::anyhow!("hsd write permission required"));
        }
        let inner = Arc::clone(&self.table.get(&value)?.inner);
        let id = inner.id.clone();
        push_ev(&events, Ev::MeshDespawned { id: id.clone() });
        if self
            .table
            .get(&self_)?
            .registry
            .doc_sync
            .load(Ordering::Relaxed)
        {
            self.table
                .get(&self_)?
                .registry
                .pending_doc_ops
                .lock()
                .expect("pending_doc_ops lock")
                .push(SyncOp::MeshRemoved(id));
        }
        Ok(())
    }

    async fn remove_material(
        &mut self,
        self_: Resource<Document>,
        value: Resource<HostMaterial>,
    ) -> wasmtime::Result<()> {
        let (events, can_write) = {
            let d = self.table.get(&self_)?;
            (Arc::clone(&d.events), d.can_write)
        };
        if !can_write {
            return Err(anyhow::anyhow!("hsd write permission required"));
        }
        let inner = Arc::clone(&self.table.get(&value)?.inner);
        let id = inner.id.clone();
        push_ev(&events, Ev::MaterialDespawned { id: id.clone() });
        if self
            .table
            .get(&self_)?
            .registry
            .doc_sync
            .load(Ordering::Relaxed)
        {
            self.table
                .get(&self_)?
                .registry
                .pending_doc_ops
                .lock()
                .expect("pending_doc_ops lock")
                .push(SyncOp::MaterialRemoved(id));
        }
        Ok(())
    }

    async fn sync(&mut self, self_: Resource<Document>) -> wasmtime::Result<bool> {
        let registry = Arc::clone(&self.table.get(&self_)?.registry);
        Ok(registry.doc_sync.load(Ordering::Relaxed))
    }

    async fn set_sync(&mut self, self_: Resource<Document>, value: bool) -> wasmtime::Result<()> {
        let (registry, can_write) = {
            let d = self.table.get(&self_)?;
            (Arc::clone(&d.registry), d.can_write)
        };
        if value && !can_write {
            return Err(anyhow::anyhow!("hsd write permission required"));
        }
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
