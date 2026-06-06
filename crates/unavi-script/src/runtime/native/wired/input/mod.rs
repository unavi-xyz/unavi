use wasmtime::component::Resource;

use crate::{
    error::ScriptError,
    permissions::ApiName,
    runtime::{
        Runtime,
        native::wired::input::bindings::InputListenerRes,
        shared::{
            self,
            wired::scene::prim::PrimRes,
        },
    },
};

mod listener;

pub mod bindings {
    pub use crate::runtime::shared::wired::{
        input::listener::InputListenerRes,
        scene::prim::PrimRes,
    };

    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-input",
        with: {
            "wired:scene/types.prim": PrimRes,
            "wired:input/types.input-listener": InputListenerRes,
            "wired:error/types": crate::runtime::native::wired::error::types,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

use crate::runtime::native::wired::error::Error;

impl bindings::wired::input::types::Host for Runtime {}

impl bindings::wired::input::api::Host for Runtime {
    async fn register_input_listener(
        &mut self,
        target: Resource<PrimRes>,
    ) -> wasmtime::Result<Result<Resource<InputListenerRes>, Error>> {
        if let Err(err) = self.api.require(ApiName::Input) {
            return Ok(Err(err.into()));
        }
        Ok(
            shared::wired::input::register_input_listener(&self.api, target.rep())
                .await
                .map(Resource::new_own)
                .map_err(|err| ScriptError::from(err).into()),
        )
    }
}

impl bindings::wired::input::context::Host for Runtime {
    async fn register_global_input_listener(
        &mut self,
    ) -> wasmtime::Result<Result<Resource<InputListenerRes>, Error>> {
        if let Err(err) = self.api.require(ApiName::InputContext) {
            return Ok(Err(err.into()));
        }
        Ok(
            shared::wired::input::register_global_input_listener(&self.api)
                .await
                .map(Resource::new_own)
                .map_err(|err| ScriptError::from(err).into()),
        )
    }
}
