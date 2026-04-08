use std::sync::{Arc, atomic::Ordering};

use bevy::mesh::PrimitiveTopology;
use bevy_hsd::cache::MeshInner;
use wasmtime::{bail, component::Resource};

use super::WiredSceneRt;
use super::bindings::wired::scene::types::{Indices, Mesh, PrimitiveTopology as WitTopology};
use crate::core_ops;

pub struct HostMesh {
    pub inner: Arc<MeshInner>,
    pub can_read: bool,
    pub can_write: bool,
}

impl Clone for HostMesh {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            can_read: self.can_read,
            can_write: self.can_write,
        }
    }
}

const fn wit_topo_to_bevy(t: WitTopology) -> PrimitiveTopology {
    match t {
        WitTopology::PointList => PrimitiveTopology::PointList,
        WitTopology::LineList => PrimitiveTopology::LineList,
        WitTopology::LineStrip => PrimitiveTopology::LineStrip,
        WitTopology::TriangleList => PrimitiveTopology::TriangleList,
        WitTopology::TriangleStrip => PrimitiveTopology::TriangleStrip,
    }
}

const fn bevy_topo_to_wit(t: PrimitiveTopology) -> WitTopology {
    match t {
        PrimitiveTopology::PointList => WitTopology::PointList,
        PrimitiveTopology::LineList => WitTopology::LineList,
        PrimitiveTopology::LineStrip => WitTopology::LineStrip,
        PrimitiveTopology::TriangleList => WitTopology::TriangleList,
        PrimitiveTopology::TriangleStrip => WitTopology::TriangleStrip,
    }
}

impl super::bindings::wired::scene::types::HostMesh for WiredSceneRt {
    async fn id(&mut self, self_: Resource<HostMesh>) -> wasmtime::Result<String> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        Ok(inner.id.to_string())
    }

    async fn clone(&mut self, self_: Resource<HostMesh>) -> wasmtime::Result<Resource<HostMesh>> {
        let inner = self.table.get(&self_)?.clone();
        Ok(self.table.push(inner)?)
    }

    async fn sync(&mut self, self_: Resource<Mesh>) -> wasmtime::Result<bool> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        Ok(inner.sync.load(Ordering::Relaxed))
    }

    async fn set_sync(&mut self, self_: Resource<Mesh>, value: bool) -> wasmtime::Result<()> {
        let (inner, can_write) = {
            let m = self.table.get(&self_)?;
            (Arc::clone(&m.inner), m.can_write)
        };
        if value && !can_write {
            bail!("hsd write permission required")
        }
        inner.sync.store(value, Ordering::Relaxed);
        Ok(())
    }

    async fn name(&mut self, self_: Resource<Mesh>) -> wasmtime::Result<Option<String>> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        Ok(inner.state.lock().expect("mesh state lock").name.clone())
    }

    async fn set_name(
        &mut self,
        self_: Resource<Mesh>,
        value: Option<String>,
    ) -> wasmtime::Result<()> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        core_ops::mesh::set_name(&inner, value);
        Ok(())
    }

    async fn topology(&mut self, self_: Resource<Mesh>) -> wasmtime::Result<WitTopology> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let state = inner.state.lock().expect("mesh state lock");
        Ok(bevy_topo_to_wit(state.topology))
    }

    async fn set_topology(
        &mut self,
        self_: Resource<Mesh>,
        value: WitTopology,
    ) -> wasmtime::Result<()> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        core_ops::mesh::set_topology(&inner, wit_topo_to_bevy(value));
        Ok(())
    }

    async fn indices(&mut self, self_: Resource<Mesh>) -> wasmtime::Result<Option<Indices>> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let state = inner.state.lock().expect("mesh state lock");
        Ok(state.indices.clone().map(Indices::Full))
    }

    async fn set_indices(
        &mut self,
        self_: Resource<Mesh>,
        value: Option<Indices>,
    ) -> wasmtime::Result<()> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let indices = value.map(|v| match v {
            Indices::Half(h) => h.into_iter().map(u32::from).collect(),
            Indices::Full(f) => f,
        });
        let mut queue = self.command_queue.lock().expect("cmd queue lock");
        core_ops::mesh::set_indices(&inner, self.doc_entity, indices, &mut queue);
        Ok(())
    }

    mesh_attr!(colors, set_colors, colors, Option<Vec<f32>>);
    mesh_attr!(normals, set_normals, normals, Option<Vec<f32>>);
    mesh_attr!(positions, set_positions, positions, Option<Vec<f32>>);
    mesh_attr!(tangents, set_tangents, tangents, Option<Vec<f32>>);
    mesh_attr!(uv0, set_uv0, uv0, Option<Vec<f32>>);
    mesh_attr!(uv1, set_uv1, uv1, Option<Vec<f32>>);

    async fn drop(&mut self, rep: Resource<Mesh>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}
