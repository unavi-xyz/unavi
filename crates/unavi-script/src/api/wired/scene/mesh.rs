use std::sync::{Arc, atomic::Ordering};

use bevy::mesh::PrimitiveTopology;
use bevy::prelude::World;
use bevy_hsd::cache::MeshInner;
use bevy_hsd::hydrate::compile::mesh::{HsdMeshGeometrySet, MeshGeometrySource};
use wasmtime::component::Resource;

use super::bindings::wired::scene::types::{Indices, Mesh, PrimitiveTopology as WitTopology};
use crate::api::wired::scene::WiredSceneRt;

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

const fn bevy_topo_to_i64(t: PrimitiveTopology) -> i64 {
    match t {
        PrimitiveTopology::PointList => 0,
        PrimitiveTopology::LineList => 1,
        PrimitiveTopology::LineStrip => 2,
        PrimitiveTopology::TriangleList => 3,
        PrimitiveTopology::TriangleStrip => 4,
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
    async fn clone(
        &mut self,
        self_: wasmtime::component::Resource<HostMesh>,
    ) -> wasmtime::Result<wasmtime::component::Resource<HostMesh>> {
        let inner = self.table.get(&self_)?.clone();
        let mesh = self.table.push(inner)?;
        Ok(mesh)
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
            return Err(anyhow::anyhow!("hsd write permission required"));
        }
        inner.sync.store(value, Ordering::Relaxed);
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
        inner
            .state
            .lock()
            .expect("mesh state lock")
            .name
            .clone_from(&value);
        if inner.sync.load(Ordering::Relaxed) {
            inner.hsd_changes.lock().expect("hsd_changes lock").name = Some(value);
        }
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
        let topo = wit_topo_to_bevy(value);
        inner.state.lock().expect("mesh state lock").topology = topo;
        if inner.sync.load(Ordering::Relaxed) {
            inner.hsd_changes.lock().expect("hsd_changes lock").topology =
                Some(bevy_topo_to_i64(topo));
        }
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
        inner.state.lock().expect("mesh state lock").indices = value.map(|v| match v {
            Indices::Half(h) => h.into_iter().map(u32::from).collect(),
            Indices::Full(f) => f,
        });
        let doc = self.doc_entity;
        let id = inner.id.clone();
        self.push_command(move |world: &mut World| {
            world.trigger(HsdMeshGeometrySet {
                doc,
                id,
                source: MeshGeometrySource::Inline,
            });
        });
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
        let doc = self.doc_entity;
        let id = inner.id.clone();
        self.push_command(move |world: &mut World| {
            world.trigger(HsdMeshGeometrySet {
                doc,
                id,
                source: MeshGeometrySource::Inline,
            });
        });
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
        let doc = self.doc_entity;
        let id = inner.id.clone();
        self.push_command(move |world: &mut World| {
            world.trigger(HsdMeshGeometrySet {
                doc,
                id,
                source: MeshGeometrySource::Inline,
            });
        });
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
        let doc = self.doc_entity;
        let id = inner.id.clone();
        self.push_command(move |world: &mut World| {
            world.trigger(HsdMeshGeometrySet {
                doc,
                id,
                source: MeshGeometrySource::Inline,
            });
        });
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
        let doc = self.doc_entity;
        let id = inner.id.clone();
        self.push_command(move |world: &mut World| {
            world.trigger(HsdMeshGeometrySet {
                doc,
                id,
                source: MeshGeometrySource::Inline,
            });
        });
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
        let doc = self.doc_entity;
        let id = inner.id.clone();
        self.push_command(move |world: &mut World| {
            world.trigger(HsdMeshGeometrySet {
                doc,
                id,
                source: MeshGeometrySource::Inline,
            });
        });
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
        let doc = self.doc_entity;
        let id = inner.id.clone();
        self.push_command(move |world: &mut World| {
            world.trigger(HsdMeshGeometrySet {
                doc,
                id,
                source: MeshGeometrySource::Inline,
            });
        });
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<Mesh>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}
