use unavi_policy::document::ApiName;
use wasmtime::component::Resource;

use crate::{
    error::ScriptError,
    runtime::{
        Runtime,
        native::wired::error::Error,
        shared::{
            self,
            wired::scene::{
                document::DocRes,
                prim::PrimRes,
            },
        },
    },
};

pub mod document;
pub mod prim;
mod shader_graph;
mod types;

pub mod bindings {
    pub use crate::runtime::shared::wired::scene::{
        document::DocRes,
        prim::PrimRes,
    };

    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-scene",
        with: {
            "wired:scene/types.document": DocRes,
            "wired:scene/types.prim":     PrimRes,
            "wired:error/types": crate::runtime::native::wired::error::types,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

impl bindings::wired::scene::api::Host for Runtime {
    async fn self_prim(&mut self) -> wasmtime::Result<Result<Resource<PrimRes>, Error>> {
        if let Err(err) = self.api.require(ApiName::Scene) {
            return Ok(Err(err.into()));
        }
        Ok(shared::wired::scene::self_prim(&self.api)
            .await
            .map(Resource::new_own)
            .map_err(|err| ScriptError::from(err).into()))
    }

    async fn self_document(&mut self) -> wasmtime::Result<Result<Resource<DocRes>, Error>> {
        if let Err(err) = self.api.require(ApiName::Scene) {
            return Ok(Err(err.into()));
        }
        Ok(shared::wired::scene::self_document(&self.api)
            .await
            .map(Resource::new_own)
            .map_err(|err| ScriptError::from(err).into()))
    }

    async fn get_document(
        &mut self,
        id: Vec<u8>,
    ) -> wasmtime::Result<Result<Option<Resource<DocRes>>, Error>> {
        if let Err(err) = self.api.require(ApiName::Scene) {
            return Ok(Err(err.into()));
        }
        Ok(shared::wired::scene::get_document(&self.api, id)
            .await
            .map(|r| r.map(Resource::new_own))
            .map_err(|err| ScriptError::from(err).into()))
    }

    async fn create_document(&mut self) -> wasmtime::Result<Result<Resource<DocRes>, Error>> {
        if let Err(err) = self.api.require(ApiName::CreateDocument) {
            return Ok(Err(err.into()));
        }
        Ok(shared::wired::scene::create_document(&self.api)
            .await
            .map(Resource::new_own)
            .map_err(Into::into))
    }

    async fn create_document_from_prefab(
        &mut self,
        prefab: Vec<u8>,
    ) -> wasmtime::Result<Result<Resource<DocRes>, Error>> {
        if let Err(err) = self.api.require(ApiName::CreateDocument) {
            return Ok(Err(err.into()));
        }
        Ok(
            shared::wired::scene::create_document_from_prefab(&self.api, prefab)
                .await
                .map(Resource::new_own)
                .map_err(Into::into),
        )
    }

    async fn remove_document(&mut self, id: Vec<u8>) -> wasmtime::Result<Result<(), Error>> {
        if let Err(err) = self.api.require(ApiName::Scene) {
            return Ok(Err(err.into()));
        }
        Ok(shared::wired::scene::remove_document(&self.api, id)
            .await
            .map_err(|err| ScriptError::from(err).into()))
    }

    async fn sync_document(&mut self, id: Vec<u8>) -> wasmtime::Result<Result<(), Error>> {
        if let Err(err) = self.api.require(ApiName::CreateDocument) {
            return Ok(Err(err.into()));
        }
        Ok(shared::wired::scene::sync_document(&self.api, id)
            .await
            .map_err(|err| ScriptError::from(err).into()))
    }

    async fn save_document(&mut self, id: Vec<u8>) -> wasmtime::Result<Result<(), Error>> {
        if let Err(err) = self.api.require(ApiName::Scene) {
            return Ok(Err(err.into()));
        }
        Ok(shared::wired::scene::save_document(&self.api, id)
            .await
            .map_err(|err| ScriptError::from(err).into()))
    }
}

impl bindings::wired::scene::types::Host for Runtime {}
