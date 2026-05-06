use wasmtime::component::Resource;

use crate::runtime::{
    Runtime,
    native::wired::scene::bindings::wired::scene::types::HostDocument,
    shared::{
        self,
        wired::scene::{document::DocRes, material::MaterialRes, mesh::MeshRes, node::NodeRes},
    },
};

impl HostDocument for Runtime {
    async fn id(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Vec<u8>> {
        shared::wired::scene::document::id(&self.api, self_.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn clone(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Resource<DocRes>> {
        shared::wired::scene::document::clone(&self.api, self_.rep())
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
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

    async fn roots(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Vec<Resource<NodeRes>>> {
        shared::wired::scene::document::roots(&self.api, self_.rep())
            .map(|v| v.into_iter().map(Resource::new_own).collect())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn nodes(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Vec<Resource<NodeRes>>> {
        shared::wired::scene::document::nodes(&self.api, self_.rep())
            .map(|v| v.into_iter().map(Resource::new_own).collect())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn create_node(
        &mut self,
        self_: Resource<DocRes>,
    ) -> wasmtime::Result<Resource<NodeRes>> {
        shared::wired::scene::document::create_node(&self.api, self_.rep())
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn remove_node(
        &mut self,
        _self_: Resource<DocRes>,
        value: Resource<NodeRes>,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::document::remove_node(&self.api, value.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn meshes(
        &mut self,
        self_: Resource<DocRes>,
    ) -> wasmtime::Result<Vec<Resource<MeshRes>>> {
        shared::wired::scene::document::meshes(&self.api, self_.rep())
            .map(|v| v.into_iter().map(Resource::new_own).collect())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn create_mesh(
        &mut self,
        self_: Resource<DocRes>,
    ) -> wasmtime::Result<Resource<MeshRes>> {
        shared::wired::scene::document::create_mesh(&self.api, self_.rep())
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn remove_mesh(
        &mut self,
        _self_: Resource<DocRes>,
        value: Resource<MeshRes>,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::document::remove_mesh(&self.api, value.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn materials(
        &mut self,
        self_: Resource<DocRes>,
    ) -> wasmtime::Result<Vec<Resource<MaterialRes>>> {
        shared::wired::scene::document::materials(&self.api, self_.rep())
            .map(|v| v.into_iter().map(Resource::new_own).collect())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn create_material(
        &mut self,
        self_: Resource<DocRes>,
    ) -> wasmtime::Result<Resource<MaterialRes>> {
        shared::wired::scene::document::create_material(&self.api, self_.rep())
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn remove_material(
        &mut self,
        _self_: Resource<DocRes>,
        value: Resource<MaterialRes>,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::document::remove_material(&self.api, value.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn drop(&mut self, rep: Resource<DocRes>) -> wasmtime::Result<()> {
        shared::wired::scene::document::on_drop(&self.api, rep.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }
}
