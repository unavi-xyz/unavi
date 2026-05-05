use wasmtime::component::Resource;

use crate::runtime::{
    Runtime,
    native::wired::input::bindings::InputListenerRes,
    shared::{self, wired::scene::node::NodeRes},
};

mod listener;

pub mod bindings {
    pub use crate::runtime::shared::wired::{
        input::listener::InputListenerRes, scene::node::NodeRes,
    };

    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-input",
        with: {
            "wired:scene/types.node": NodeRes,
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
        target: Resource<NodeRes>,
    ) -> wasmtime::Result<Resource<InputListenerRes>> {
        let res = shared::wired::input::register_input_listener(&self.api, target.rep())
            .map_err(wasmtime::Error::from_anyhow)?;
        Ok(Resource::new_own(res))
    }
}

impl bindings::wired::input::context::Host for Runtime {
    async fn register_global_input_listener(
        &mut self,
    ) -> wasmtime::Result<Resource<InputListenerRes>> {
        let res = shared::wired::input::register_global_input_listener(&self.api)
            .map_err(wasmtime::Error::from_anyhow)?;
        Ok(Resource::new_own(res))
    }
}
