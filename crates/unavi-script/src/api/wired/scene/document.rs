use std::sync::Arc;
use std::sync::atomic::Ordering;

use bevy_hsd::cache::{MaterialInner, MeshInner, NodeInner, SceneRegistryInner};
use wasmtime::bail;
use wasmtime::component::Resource;

use super::bindings::wired::scene::types::{Document, Material, Mesh};
use crate::api::wired::scene::{
    WiredSceneRt, material::HostMaterial, mesh::HostMesh, node::HostNode,
};
use crate::core_ops;

pub struct HostDocument {
    pub id: blake3::Hash,
    pub registry: Arc<SceneRegistryInner>,
    pub doc_entity: bevy::prelude::Entity,
    pub can_read: bool,
    pub can_write: bool,
}

impl Clone for HostDocument {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            registry: Arc::clone(&self.registry),
            doc_entity: self.doc_entity,
            can_read: self.can_read,
            can_write: self.can_write,
        }
    }
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
        let (doc_entity, registry, can_write) = {
            let d = self.table.get(&self_)?;
            (d.doc_entity, Arc::clone(&d.registry), d.can_write)
        };
        if !can_write {
            bail!("hsd write permission required")
        }
        let mut queue = self.command_queue.lock().expect("cmd queue lock");
        let inner = core_ops::document::create_material(&registry, doc_entity, &mut queue);
        drop(queue);
        Ok(self.table.push(HostMaterial {
            inner,
            can_read: true,
            can_write: true,
        })?)
    }

    async fn create_mesh(&mut self, self_: Resource<Document>) -> wasmtime::Result<Resource<Mesh>> {
        let (doc_entity, registry, can_write) = {
            let d = self.table.get(&self_)?;
            (d.doc_entity, Arc::clone(&d.registry), d.can_write)
        };
        if !can_write {
            bail!("hsd write permission required")
        }
        let mut queue = self.command_queue.lock().expect("cmd queue lock");
        let inner = core_ops::document::create_mesh(&registry, doc_entity, &mut queue);
        drop(queue);
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
        let (doc_entity, registry, can_write) = {
            let d = self.table.get(&self_)?;
            (d.doc_entity, Arc::clone(&d.registry), d.can_write)
        };
        if !can_write {
            bail!("hsd write permission required")
        }
        let mut queue = self.command_queue.lock().expect("cmd queue lock");
        let inner = core_ops::document::create_node(&registry, doc_entity, &mut queue);
        drop(queue);
        Ok(self.table.push(HostNode {
            inner,
            can_read: true,
            can_write: true,
            doc_entity,
        })?)
    }

    async fn roots(
        &mut self,
        self_: Resource<Document>,
    ) -> wasmtime::Result<Vec<Resource<HostNode>>> {
        let (registry, can_read, can_write, doc_entity) = {
            let d = self.table.get(&self_)?;
            (
                Arc::clone(&d.registry),
                d.can_read,
                d.can_write,
                d.doc_entity,
            )
        };
        if !can_read {
            bail!("hsd read permission required")
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
                doc_entity,
            })?);
        }
        Ok(out)
    }

    async fn nodes(
        &mut self,
        self_: Resource<Document>,
    ) -> wasmtime::Result<Vec<Resource<HostNode>>> {
        let (registry, can_read, can_write, doc_entity) = {
            let d = self.table.get(&self_)?;
            (
                Arc::clone(&d.registry),
                d.can_read,
                d.can_write,
                d.doc_entity,
            )
        };
        if !can_read {
            bail!("hsd read permission required")
        }
        let nodes: Vec<Arc<NodeInner>> = registry.nodes.lock().expect("nodes lock").clone();
        let mut out = Vec::with_capacity(nodes.len());
        for inner in nodes {
            out.push(self.table.push(HostNode {
                inner,
                can_read,
                can_write,
                doc_entity,
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
            bail!("hsd read permission required")
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
            bail!("hsd read permission required")
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
        let (doc_entity, registry, can_write) = {
            let d = self.table.get(&self_)?;
            (d.doc_entity, Arc::clone(&d.registry), d.can_write)
        };
        if !can_write {
            bail!("hsd write permission required")
        }
        let inner = Arc::clone(&self.table.get(&value)?.inner);
        let mut queue = self.command_queue.lock().expect("cmd queue lock");
        core_ops::document::remove_node(&inner, &registry, doc_entity, &mut queue);
        Ok(())
    }

    async fn remove_mesh(
        &mut self,
        self_: Resource<Document>,
        value: Resource<HostMesh>,
    ) -> wasmtime::Result<()> {
        let (doc_entity, registry, can_write) = {
            let d = self.table.get(&self_)?;
            (d.doc_entity, Arc::clone(&d.registry), d.can_write)
        };
        if !can_write {
            bail!("hsd write permission required")
        }
        let inner = Arc::clone(&self.table.get(&value)?.inner);
        let mut queue = self.command_queue.lock().expect("cmd queue lock");
        core_ops::document::remove_mesh(&inner, &registry, doc_entity, &mut queue);
        Ok(())
    }

    async fn remove_material(
        &mut self,
        self_: Resource<Document>,
        value: Resource<HostMaterial>,
    ) -> wasmtime::Result<()> {
        let (doc_entity, registry, can_write) = {
            let d = self.table.get(&self_)?;
            (d.doc_entity, Arc::clone(&d.registry), d.can_write)
        };
        if !can_write {
            bail!("hsd write permission required")
        }
        let inner = Arc::clone(&self.table.get(&value)?.inner);
        let mut queue = self.command_queue.lock().expect("cmd queue lock");
        core_ops::document::remove_material(&inner, &registry, doc_entity, &mut queue);
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
            bail!("hsd write permission required")
        }
        core_ops::document::set_sync(&registry, value);
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
