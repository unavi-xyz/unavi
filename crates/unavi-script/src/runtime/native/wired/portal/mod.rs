use unavi_policy::document::ApiName;
use wasmtime::component::Resource;

use crate::runtime::{
    Runtime,
    shared::{
        self,
        wired::scene::prim::PrimRes,
    },
};

pub mod bindings {
    pub use crate::runtime::shared::wired::scene::prim::PrimRes;

    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-portal",
        with: {
            "wired:scene/types.prim": PrimRes,
            "wired:error/types": crate::runtime::native::wired::error::types,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

use crate::runtime::native::wired::error::Error;

impl bindings::wired::portal::api::Host for Runtime {
    async fn open(
        &mut self,
        prim: Resource<PrimRes>,
        target_space: Vec<u8>,
    ) -> wasmtime::Result<Result<(), Error>> {
        let result = match self.api.require(ApiName::Portal) {
            Ok(()) => shared::wired::portal::open(&self.api, prim.rep(), target_space).await,
            Err(err) => Err(err),
        };
        Ok(result.map_err(Into::into))
    }

    async fn travel(&mut self, target_space: Vec<u8>) -> wasmtime::Result<Result<(), Error>> {
        let result = match self.api.require(ApiName::Travel) {
            Ok(()) => shared::wired::portal::travel(&self.api, target_space).await,
            Err(err) => Err(err),
        };
        Ok(result.map_err(Into::into))
    }
}
