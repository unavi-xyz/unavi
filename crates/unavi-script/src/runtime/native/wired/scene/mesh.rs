use wasmtime::component::Resource;

use crate::runtime::{
    Runtime,
    native::wired::scene::bindings::wired::scene::types::{HostMesh, Indices, PrimitiveTopology},
    shared::{
        self,
        wired::scene::mesh::{MeshIndices, MeshRes, MeshTopology},
    },
};

const fn wit_to_topology(t: PrimitiveTopology) -> MeshTopology {
    match t {
        PrimitiveTopology::PointList => MeshTopology::PointList,
        PrimitiveTopology::LineList => MeshTopology::LineList,
        PrimitiveTopology::LineStrip => MeshTopology::LineStrip,
        PrimitiveTopology::TriangleList => MeshTopology::TriangleList,
        PrimitiveTopology::TriangleStrip => MeshTopology::TriangleStrip,
    }
}

const fn topology_to_wit(t: MeshTopology) -> PrimitiveTopology {
    match t {
        MeshTopology::PointList => PrimitiveTopology::PointList,
        MeshTopology::LineList => PrimitiveTopology::LineList,
        MeshTopology::LineStrip => PrimitiveTopology::LineStrip,
        MeshTopology::TriangleList => PrimitiveTopology::TriangleList,
        MeshTopology::TriangleStrip => PrimitiveTopology::TriangleStrip,
    }
}

fn wit_to_indices(i: Indices) -> MeshIndices {
    match i {
        Indices::Half(v) => MeshIndices::Half(v),
        Indices::Full(v) => MeshIndices::Full(v),
    }
}

fn indices_to_wit(i: MeshIndices) -> Indices {
    match i {
        MeshIndices::Half(v) => Indices::Half(v),
        MeshIndices::Full(v) => Indices::Full(v),
    }
}

impl HostMesh for Runtime {
    async fn id(&mut self, self_: Resource<MeshRes>) -> wasmtime::Result<String> {
        shared::wired::scene::mesh::id(&self.api, self_.rep()).map_err(wasmtime::Error::from_anyhow)
    }

    async fn clone(&mut self, self_: Resource<MeshRes>) -> wasmtime::Result<Resource<MeshRes>> {
        shared::wired::scene::mesh::clone(&self.api, self_.rep())
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn name(&mut self, self_: Resource<MeshRes>) -> wasmtime::Result<Option<String>> {
        shared::wired::scene::mesh::name(&self.api, self_.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn set_name(
        &mut self,
        self_: Resource<MeshRes>,
        value: Option<String>,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::mesh::set_name(&self.api, self_.rep(), value)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn topology(&mut self, self_: Resource<MeshRes>) -> wasmtime::Result<PrimitiveTopology> {
        shared::wired::scene::mesh::topology(&self.api, self_.rep())
            .map(topology_to_wit)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn set_topology(
        &mut self,
        self_: Resource<MeshRes>,
        value: PrimitiveTopology,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::mesh::set_topology(&self.api, self_.rep(), wit_to_topology(value))
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn indices(&mut self, self_: Resource<MeshRes>) -> wasmtime::Result<Option<Indices>> {
        shared::wired::scene::mesh::indices(&self.api, self_.rep())
            .await
            .map(|opt| opt.map(indices_to_wit))
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn set_indices(
        &mut self,
        self_: Resource<MeshRes>,
        value: Option<Indices>,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::mesh::set_indices(&self.api, self_.rep(), value.map(wit_to_indices))
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn positions(&mut self, self_: Resource<MeshRes>) -> wasmtime::Result<Option<Vec<f32>>> {
        shared::wired::scene::mesh::positions(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn set_positions(
        &mut self,
        self_: Resource<MeshRes>,
        values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::mesh::set_positions(&self.api, self_.rep(), values)
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn normals(&mut self, self_: Resource<MeshRes>) -> wasmtime::Result<Option<Vec<f32>>> {
        shared::wired::scene::mesh::normals(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn set_normals(
        &mut self,
        self_: Resource<MeshRes>,
        values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::mesh::set_normals(&self.api, self_.rep(), values)
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn tangents(&mut self, self_: Resource<MeshRes>) -> wasmtime::Result<Option<Vec<f32>>> {
        shared::wired::scene::mesh::tangents(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn set_tangents(
        &mut self,
        self_: Resource<MeshRes>,
        values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::mesh::set_tangents(&self.api, self_.rep(), values)
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn colors(&mut self, self_: Resource<MeshRes>) -> wasmtime::Result<Option<Vec<f32>>> {
        shared::wired::scene::mesh::colors(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn set_colors(
        &mut self,
        self_: Resource<MeshRes>,
        values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::mesh::set_colors(&self.api, self_.rep(), values)
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn uv0(&mut self, self_: Resource<MeshRes>) -> wasmtime::Result<Option<Vec<f32>>> {
        shared::wired::scene::mesh::uv0(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn set_uv0(
        &mut self,
        self_: Resource<MeshRes>,
        values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::mesh::set_uv0(&self.api, self_.rep(), values)
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn uv1(&mut self, self_: Resource<MeshRes>) -> wasmtime::Result<Option<Vec<f32>>> {
        shared::wired::scene::mesh::uv1(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn set_uv1(
        &mut self,
        self_: Resource<MeshRes>,
        values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::mesh::set_uv1(&self.api, self_.rep(), values)
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn drop(&mut self, rep: Resource<MeshRes>) -> wasmtime::Result<()> {
        shared::wired::scene::mesh::on_drop(&self.api, rep.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }
}
