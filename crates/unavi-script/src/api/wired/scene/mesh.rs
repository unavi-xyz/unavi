use std::sync::{Arc, atomic::Ordering};

use bevy::mesh::PrimitiveTopology;
use bevy_hsd::cache::MeshInner;
use wasmtime::component::Resource;

use super::bindings::wired::scene::types::{Indices, Mesh, PrimitiveTopology as WitTopology};
use crate::api::wired::scene::WiredSceneRt;

pub struct HostMesh {
    pub inner: Arc<MeshInner>,
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
    async fn id(
        &mut self,
        self_: wasmtime::component::Resource<HostMesh>,
    ) -> wasmtime::Result<String> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        Ok(inner.id.to_string())
    }

    async fn commit(&mut self, self_: Resource<Mesh>) -> wasmtime::Result<()> {
        self.check_hsd_write()?;
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let state = inner.state.lock().expect("mesh state lock");
        let map = self.mesh_map(&inner.id)?;
        let topo: i64 = match state.topology {
            PrimitiveTopology::PointList => 0,
            PrimitiveTopology::LineList => 1,
            PrimitiveTopology::LineStrip => 2,
            PrimitiveTopology::TriangleList => 3,
            PrimitiveTopology::TriangleStrip => 4,
        };
        map.insert("topology", topo)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        if let Some(name) = &state.name {
            map.insert("name", name.as_str())
                .map_err(|e| anyhow::anyhow!("{e}"))?;
        }
        drop(state);
        self.doc.commit();
        self.doc.compact_change_store();
        self.doc.free_history_cache();
        Ok(())
    }

    async fn name(&mut self, self_: Resource<Mesh>) -> wasmtime::Result<Option<String>> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        let state = inner.state.lock().expect("mesh state lock");
        Ok(state.name.clone())
    }

    async fn set_name(
        &mut self,
        self_: Resource<Mesh>,
        value: Option<String>,
    ) -> wasmtime::Result<()> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        inner.state.lock().expect("mesh state lock").name = value;
        inner.dirty.store(true, Ordering::Relaxed);
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
        inner.state.lock().expect("mesh state lock").topology = wit_topo_to_bevy(value);
        inner.dirty.store(true, Ordering::Relaxed);
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
        {
            let mut state = inner.state.lock().expect("mesh state lock");
            state.indices = value.map(|v| match v {
                Indices::Half(h) => h.into_iter().map(u32::from).collect(),
                Indices::Full(f) => f,
            });
        }
        inner.dirty.store(true, Ordering::Relaxed);
        Ok(())
    }

    async fn colors(&mut self, self_: Resource<Mesh>) -> wasmtime::Result<Option<Vec<f32>>> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        Ok(inner.state.lock().expect("mesh state lock").colors.clone())
    }

    async fn set_colors(
        &mut self,
        self_: Resource<Mesh>,
        values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        inner.state.lock().expect("mesh state lock").colors = values;
        inner.dirty.store(true, Ordering::Relaxed);
        Ok(())
    }

    async fn normals(&mut self, self_: Resource<Mesh>) -> wasmtime::Result<Option<Vec<f32>>> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        Ok(inner.state.lock().expect("mesh state lock").normals.clone())
    }

    async fn set_normals(
        &mut self,
        self_: Resource<Mesh>,
        values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        inner.state.lock().expect("mesh state lock").normals = values;
        inner.dirty.store(true, Ordering::Relaxed);
        Ok(())
    }

    async fn positions(&mut self, self_: Resource<Mesh>) -> wasmtime::Result<Option<Vec<f32>>> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        Ok(inner
            .state
            .lock()
            .expect("mesh state lock")
            .positions
            .clone())
    }

    async fn set_positions(
        &mut self,
        self_: Resource<Mesh>,
        values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        inner.state.lock().expect("mesh state lock").positions = values;
        inner.dirty.store(true, Ordering::Relaxed);
        Ok(())
    }

    async fn tangents(&mut self, self_: Resource<Mesh>) -> wasmtime::Result<Option<Vec<f32>>> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        Ok(inner
            .state
            .lock()
            .expect("mesh state lock")
            .tangents
            .clone())
    }

    async fn set_tangents(
        &mut self,
        self_: Resource<Mesh>,
        values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        inner.state.lock().expect("mesh state lock").tangents = values;
        inner.dirty.store(true, Ordering::Relaxed);
        Ok(())
    }

    async fn uv0(&mut self, self_: Resource<Mesh>) -> wasmtime::Result<Option<Vec<f32>>> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        Ok(inner.state.lock().expect("mesh state lock").uv0.clone())
    }

    async fn set_uv0(
        &mut self,
        self_: Resource<Mesh>,
        values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        inner.state.lock().expect("mesh state lock").uv0 = values;
        inner.dirty.store(true, Ordering::Relaxed);
        Ok(())
    }

    async fn uv1(&mut self, self_: Resource<Mesh>) -> wasmtime::Result<Option<Vec<f32>>> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        Ok(inner.state.lock().expect("mesh state lock").uv1.clone())
    }

    async fn set_uv1(
        &mut self,
        self_: Resource<Mesh>,
        values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        let inner = Arc::clone(&self.table.get(&self_)?.inner);
        inner.state.lock().expect("mesh state lock").uv1 = values;
        inner.dirty.store(true, Ordering::Relaxed);
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<Mesh>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}
