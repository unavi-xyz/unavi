use unavi_policy::document::ApiName;
use wasmtime::component::Resource;

use crate::{
    error::ScriptError,
    runtime::{
        Runtime,
        native::wired::input::bindings::{
            InputListenerRes,
            PointerClaimRes,
            wired::input::types::{
                HostPointerClaim,
                Pointer,
                PointerKind as WitPointerKind,
            },
        },
        shared::{
            self,
            wired::scene::prim::PrimRes,
        },
    },
};

mod listener;
mod types;

pub mod bindings {
    pub use crate::runtime::shared::wired::{
        input::{
            PointerClaimRes,
            listener::InputListenerRes,
        },
        scene::prim::PrimRes,
    };

    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-input",
        with: {
            "wired:scene/types.prim": PrimRes,
            "wired:input/types.input-listener": InputListenerRes,
            "wired:input/types.pointer-claim": PointerClaimRes,
            "wired:error/types": crate::runtime::native::wired::error::bindings::wired::error::types,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

use crate::runtime::native::wired::error::bindings::wired::error::types::Error;

impl bindings::wired::input::types::Host for Runtime {}

impl HostPointerClaim for Runtime {
    async fn kind(&mut self, self_: Resource<PointerClaimRes>) -> wasmtime::Result<WitPointerKind> {
        let kind = shared::wired::input::claimed_kind(self_.rep())
            .ok_or_else(|| wasmtime::Error::msg("claim does not name a pointer"))?;
        Ok(kind.into())
    }

    async fn drop(&mut self, rep: Resource<PointerClaimRes>) -> wasmtime::Result<()> {
        shared::wired::input::release_pointer(rep.rep());
        Ok(())
    }
}

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

    async fn pointers(&mut self) -> wasmtime::Result<Result<Vec<Pointer>, Error>> {
        if let Err(err) = self.api.require(ApiName::InputContext) {
            return Ok(Err(err.into()));
        }
        Ok(Ok(shared::wired::input::pointers()
            .into_iter()
            .map(Into::into)
            .collect()))
    }

    async fn claim_pointer(
        &mut self,
        kind: WitPointerKind,
    ) -> wasmtime::Result<Result<Resource<PointerClaimRes>, Error>> {
        if let Err(err) = self.api.require(ApiName::InputContext) {
            return Ok(Err(err.into()));
        }
        Ok(
            shared::wired::input::claim_pointer(self.api.doc_id, kind.into())
                .map(Resource::new_own)
                .map_err(|err| ScriptError::from(err).into()),
        )
    }
}
