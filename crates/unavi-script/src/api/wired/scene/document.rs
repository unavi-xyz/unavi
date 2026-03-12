use std::sync::{Arc, Mutex};

use bevy_hsd::{
    MaterialInner, MaterialState, MeshInner, MeshState, NodeInner, NodeState, SceneEvent,
};
use loro::{LoroMap, LoroValue, TreeParentId};
use rand::{Rng, distr::Alphanumeric};
use smol_str::SmolStr;
use wasmtime::component::Resource;

use super::bindings::wired::scene::types::{Document, Material, Mesh};
use crate::api::wired::scene::{
    WiredSceneRt, material::HostMaterial, mesh::HostMesh, node::HostNode,
};

pub struct HostDocument {
    pub id: blake3::Hash,
}

fn gen_id() -> SmolStr {
    const MAX_INLINE: usize = 23;

    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(MAX_INLINE)
        .map(char::from)
        .collect::<SmolStr>()
}

impl super::bindings::wired::scene::types::HostDocument for WiredSceneRt {
    async fn create_material(
        &mut self,
        _self_: Resource<Document>,
    ) -> wasmtime::Result<Resource<Material>> {
        // Create Loro entry to get stable id.
        let id = gen_id();
        let map: LoroMap = self
            .materials()?
            .insert_container(&id, LoroMap::new())
            .map_err(|e| anyhow::anyhow!("push material: {e}"))?;
        map.insert("base_color", LoroValue::Null)
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        // Create cache entry.
        let inner = Arc::new(MaterialInner {
            id: id.clone(),
            state: Mutex::new(MaterialState::default()),
            entity: Mutex::new(None),
        });
        {
            let mut mats = self.registry.materials.lock().expect("materials lock");
            mats.insert(id, Arc::clone(&inner));
        }
        self.push_event(SceneEvent::MaterialCreated(Arc::clone(&inner)));

        Ok(self.table.push(HostMaterial { inner })?)
    }

    async fn create_mesh(
        &mut self,
        _self_: Resource<Document>,
    ) -> wasmtime::Result<Resource<Mesh>> {
        // Create Loro entry to get stable id.
        let id = gen_id();
        let map: LoroMap = self
            .meshes()?
            .insert_container(&id, LoroMap::new())
            .map_err(|e| anyhow::anyhow!("push mesh: {e}"))?;
        map.insert("topology", 3i64) // TODO set from enum
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        // Create cache entry.
        let inner = Arc::new(MeshInner {
            id: id.clone(),
            state: Mutex::new(MeshState::default()),
            entity: Mutex::new(None),
        });
        {
            let mut meshes = self.registry.meshes.lock().expect("meshes lock");
            meshes.insert(id, Arc::clone(&inner));
        }
        self.push_event(SceneEvent::MeshCreated(Arc::clone(&inner)));

        Ok(self.table.push(HostMesh { inner })?)
    }

    async fn create_node(
        &mut self,
        _self_: Resource<Document>,
    ) -> wasmtime::Result<Resource<HostNode>> {
        // Create Loro tree node to get stable TreeID.
        let tree_id = self
            .nodes()?
            .create(TreeParentId::Root)
            .map_err(|e| anyhow::anyhow!("create node: {e}"))?;

        // Create cache entry.
        let inner = Arc::new(NodeInner {
            tree_id,
            state: Mutex::new(NodeState::default()),
            entity: Mutex::new(None),
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
            .insert(tree_id, Arc::clone(&inner));
        self.push_event(SceneEvent::NodeCreated(Arc::clone(&inner)));

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

    async fn id(&mut self, self_: Resource<Document>) -> wasmtime::Result<Vec<u8>> {
        Ok(self.table.get(&self_)?.id.as_bytes().to_vec())
    }

    async fn commit(&mut self, _self_: Resource<Document>) -> wasmtime::Result<()> {
        self.check_hsd_write()?;
        self.doc.commit();
        self.doc.compact_change_store();
        self.doc.free_history_cache();
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<Document>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}
