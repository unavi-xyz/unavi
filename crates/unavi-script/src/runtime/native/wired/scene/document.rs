use wasmtime::component::Resource;

use crate::runtime::{
    Runtime,
    native::wired::scene::bindings::wired::{
        math::types::{Quat, Transform, Vec3},
        scene::types::HostDocument,
    },
    shared::{
        self,
        wired::scene::{document::DocRes, material::MaterialRes, mesh::MeshRes, node::NodeRes},
    },
};

impl HostDocument for Runtime {
    async fn id(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Vec<u8>> {
        shared::wired::scene::document::doc_id(&self.backend, self_.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn clone(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Resource<DocRes>> {
        shared::wired::scene::document::doc_clone(&self.backend, self_.rep())
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn roots(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Vec<Resource<NodeRes>>> {
        shared::wired::scene::document::doc_roots(&self.backend, self_.rep())
            .await
            .map(|v| v.into_iter().map(Resource::new_own).collect())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn nodes(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Vec<Resource<NodeRes>>> {
        shared::wired::scene::document::doc_nodes(&self.backend, self_.rep())
            .await
            .map(|v| v.into_iter().map(Resource::new_own).collect())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn meshes(
        &mut self,
        self_: Resource<DocRes>,
    ) -> wasmtime::Result<Vec<Resource<MeshRes>>> {
        shared::wired::scene::document::doc_meshes(&self.backend, self_.rep())
            .await
            .map(|v| v.into_iter().map(Resource::new_own).collect())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn materials(
        &mut self,
        self_: Resource<DocRes>,
    ) -> wasmtime::Result<Vec<Resource<MaterialRes>>> {
        shared::wired::scene::document::doc_materials(&self.backend, self_.rep())
            .await
            .map(|v| v.into_iter().map(Resource::new_own).collect())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn create_node(
        &mut self,
        self_: Resource<DocRes>,
    ) -> wasmtime::Result<Resource<NodeRes>> {
        shared::wired::scene::document::doc_create_node(&self.backend, self_.rep())
            .await
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn create_mesh(
        &mut self,
        self_: Resource<DocRes>,
    ) -> wasmtime::Result<Resource<MeshRes>> {
        shared::wired::scene::document::doc_create_mesh(&self.backend, self_.rep())
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn create_material(
        &mut self,
        self_: Resource<DocRes>,
    ) -> wasmtime::Result<Resource<MaterialRes>> {
        shared::wired::scene::document::doc_create_material(&self.backend, self_.rep())
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn remove_node(
        &mut self,
        _self_: Resource<DocRes>,
        value: Resource<NodeRes>,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::document::doc_remove_node(&self.backend, value.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn remove_mesh(
        &mut self,
        _self_: Resource<DocRes>,
        value: Resource<MeshRes>,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::document::doc_remove_mesh(&self.backend, value.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn remove_material(
        &mut self,
        _self_: Resource<DocRes>,
        value: Resource<MaterialRes>,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::document::doc_remove_material(&self.backend, value.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn sync(&mut self, _self_: Resource<DocRes>) -> wasmtime::Result<bool> {
        Ok(false)
    }

    async fn set_sync(&mut self, _self_: Resource<DocRes>, _value: bool) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn public(&mut self, _self_: Resource<DocRes>) -> wasmtime::Result<bool> {
        Ok(false)
    }

    async fn set_public(&mut self, _self_: Resource<DocRes>, _value: bool) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn assets(
        &mut self,
        _self_: Resource<DocRes>,
    ) -> wasmtime::Result<Vec<(String, Vec<u8>)>> {
        Ok(vec![])
    }

    async fn add_asset(
        &mut self,
        _self_: Resource<DocRes>,
        _name: String,
        _blob_id: Vec<u8>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn remove_asset(
        &mut self,
        _self_: Resource<DocRes>,
        _name: String,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn translation(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Vec3> {
        Ok(shared::wired::scene::document::doc_translation(&self.backend, self_.rep()).into())
    }

    async fn set_translation(
        &mut self,
        self_: Resource<DocRes>,
        value: Vec3,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::document::doc_set_translation(
            &self.backend,
            self_.rep(),
            value.into(),
        )
        .map_err(wasmtime::Error::from_anyhow)
    }

    async fn rotation(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Quat> {
        Ok(shared::wired::scene::document::doc_rotation(&self.backend, self_.rep()).into())
    }

    async fn set_rotation(&mut self, self_: Resource<DocRes>, value: Quat) -> wasmtime::Result<()> {
        shared::wired::scene::document::doc_set_rotation(&self.backend, self_.rep(), value.into())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn scale(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Vec3> {
        Ok(shared::wired::scene::document::doc_scale(&self.backend, self_.rep()).into())
    }

    async fn set_scale(&mut self, self_: Resource<DocRes>, value: Vec3) -> wasmtime::Result<()> {
        shared::wired::scene::document::doc_set_scale(&self.backend, self_.rep(), value.into())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn transform(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Transform> {
        Ok(shared::wired::scene::document::doc_transform(&self.backend, self_.rep()).into())
    }

    async fn set_transform(
        &mut self,
        self_: Resource<DocRes>,
        value: Transform,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::document::doc_set_transform(&self.backend, self_.rep(), value.into())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn global_transform(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Transform> {
        Ok(shared::wired::scene::document::doc_global_transform(&self.backend, self_.rep()).into())
    }

    async fn drop(&mut self, rep: Resource<DocRes>) -> wasmtime::Result<()> {
        shared::wired::scene::document::doc_drop(&self.backend, rep.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }
}
