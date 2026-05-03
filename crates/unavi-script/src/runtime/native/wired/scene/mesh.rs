use wasmtime::component::Resource;

use crate::runtime::{
    Runtime,
    native::wired::scene::bindings::wired::scene::types::{HostMesh, Indices, PrimitiveTopology},
    shared::wired::scene::mesh::MeshRes,
};

impl HostMesh for Runtime {
    async fn id(&mut self, _self_: Resource<MeshRes>) -> wasmtime::Result<String> {
        Ok(String::new())
    }

    async fn clone(&mut self, self_: Resource<MeshRes>) -> wasmtime::Result<Resource<MeshRes>> {
        let rep = self
            .backend
            .wired_scene
            .lock()
            .await
            .mesh_clone(self_.rep())
            .ok_or_else(|| wasmtime::Error::msg("invalid mesh"))?;
        Ok(Resource::new_own(rep))
    }

    async fn name(&mut self, _self_: Resource<MeshRes>) -> wasmtime::Result<Option<String>> {
        Ok(None)
    }

    async fn set_name(
        &mut self,
        _self_: Resource<MeshRes>,
        _value: Option<String>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn topology(&mut self, _self_: Resource<MeshRes>) -> wasmtime::Result<PrimitiveTopology> {
        Ok(PrimitiveTopology::TriangleList)
    }

    async fn set_topology(
        &mut self,
        _self_: Resource<MeshRes>,
        _value: PrimitiveTopology,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn indices(&mut self, _self_: Resource<MeshRes>) -> wasmtime::Result<Option<Indices>> {
        Ok(None)
    }

    async fn set_indices(
        &mut self,
        _self_: Resource<MeshRes>,
        _value: Option<Indices>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn positions(&mut self, _self_: Resource<MeshRes>) -> wasmtime::Result<Option<Vec<f32>>> {
        Ok(None)
    }

    async fn set_positions(
        &mut self,
        _self_: Resource<MeshRes>,
        _values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn normals(&mut self, _self_: Resource<MeshRes>) -> wasmtime::Result<Option<Vec<f32>>> {
        Ok(None)
    }

    async fn set_normals(
        &mut self,
        _self_: Resource<MeshRes>,
        _values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn tangents(&mut self, _self_: Resource<MeshRes>) -> wasmtime::Result<Option<Vec<f32>>> {
        Ok(None)
    }

    async fn set_tangents(
        &mut self,
        _self_: Resource<MeshRes>,
        _values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn colors(&mut self, _self_: Resource<MeshRes>) -> wasmtime::Result<Option<Vec<f32>>> {
        Ok(None)
    }

    async fn set_colors(
        &mut self,
        _self_: Resource<MeshRes>,
        _values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn uv0(&mut self, _self_: Resource<MeshRes>) -> wasmtime::Result<Option<Vec<f32>>> {
        Ok(None)
    }

    async fn set_uv0(
        &mut self,
        _self_: Resource<MeshRes>,
        _values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn uv1(&mut self, _self_: Resource<MeshRes>) -> wasmtime::Result<Option<Vec<f32>>> {
        Ok(None)
    }

    async fn set_uv1(
        &mut self,
        _self_: Resource<MeshRes>,
        _values: Option<Vec<f32>>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn sync(&mut self, _self_: Resource<MeshRes>) -> wasmtime::Result<bool> {
        Ok(false)
    }

    async fn set_sync(&mut self, _self_: Resource<MeshRes>, _value: bool) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<MeshRes>) -> wasmtime::Result<()> {
        self.backend
            .wired_scene
            .lock()
            .await
            .meshes
            .remove(rep.rep());
        Ok(())
    }
}
