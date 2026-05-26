use wasmtime::component::Resource;

use crate::runtime::{
    Runtime,
    native::wired::input::bindings::InputListenerRes,
    shared::{self, wired::scene::prim::PrimRes},
};

mod listener;

pub mod bindings {
    pub use crate::runtime::shared::wired::{
        input::listener::InputListenerRes, scene::prim::PrimRes,
    };

    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-input",
        with: {
            "wired:scene/types.prim": PrimRes,
            "wired:input/types.input-listener": InputListenerRes,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

impl bindings::wired::input::types::Host for Runtime {}

impl bindings::wired::input::api::Host for Runtime {
    async fn register_input_listener(
        &mut self,
        target: Resource<PrimRes>,
    ) -> wasmtime::Result<Resource<InputListenerRes>> {
        let res = shared::wired::input::register_input_listener(&self.api, target.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)?;
        Ok(Resource::new_own(res))
    }
}

impl bindings::wired::input::context::Host for Runtime {
    async fn register_global_input_listener(
        &mut self,
    ) -> wasmtime::Result<Resource<InputListenerRes>> {
        let res = shared::wired::input::register_global_input_listener(&self.api)
            .await
            .map_err(wasmtime::Error::from_anyhow)?;
        Ok(Resource::new_own(res))
    }
}
