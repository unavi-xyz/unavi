use wasmtime::component::Resource;

use crate::runtime::{
    Runtime,
    native::wired::scene::bindings::wired::scene::types::HostDocument,
    shared::{
        self,
        wired::scene::{
            document::DocRes,
            prim::PrimRes,
        },
    },
};

impl HostDocument for Runtime {
    async fn id(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Vec<u8>> {
        shared::wired::scene::document::id(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn clone(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Resource<DocRes>> {
        shared::wired::scene::document::clone(&self.api, self_.rep())
            .await
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn roots(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Vec<Resource<PrimRes>>> {
        shared::wired::scene::document::roots(&self.api, self_.rep())
            .await
            .map(|v| v.into_iter().map(Resource::new_own).collect())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn prims(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Vec<Resource<PrimRes>>> {
        shared::wired::scene::document::prims(&self.api, self_.rep())
            .await
            .map(|v| v.into_iter().map(Resource::new_own).collect())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn get_prim(
        &mut self,
        self_: Resource<DocRes>,
        id: String,
    ) -> wasmtime::Result<Option<Resource<PrimRes>>> {
        shared::wired::scene::document::get_prim(&self.api, self_.rep(), id)
            .await
            .map(|r| r.map(Resource::new_own))
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn create_prim(
        &mut self,
        self_: Resource<DocRes>,
    ) -> wasmtime::Result<Resource<PrimRes>> {
        shared::wired::scene::document::create_prim(&self.api, self_.rep())
            .await
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn remove_prim(
        &mut self,
        _self_: Resource<DocRes>,
        value: Resource<PrimRes>,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::document::remove_prim(&self.api, value.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn drop(&mut self, rep: Resource<DocRes>) -> wasmtime::Result<()> {
        shared::wired::scene::document::on_drop(&self.api, rep.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }
}
