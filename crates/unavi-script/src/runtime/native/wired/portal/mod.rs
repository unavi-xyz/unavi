use wasmtime::component::Resource;

use crate::runtime::{
    Runtime,
    shared::{
        self,
        wired::portal::{
            PortalDestination,
            PortalParams,
            PortalRes,
            PortalTransform,
        },
    },
};

pub mod bindings {
    pub use crate::runtime::shared::wired::portal::PortalRes;

    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-portal",
        with: {
            "wired:portal/types.portal": PortalRes,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

use bindings::wired::portal::{
    api::{
        Portal as ApiPortal,
        PortalParams as WitParams,
    },
    types::{
        HostPortal,
        PortalDestination as WitDest,
    },
};

impl From<WitDest> for PortalDestination {
    fn from(d: WitDest) -> Self {
        Self {
            space:  d.space,
            portal: d.portal,
        }
    }
}

impl From<PortalDestination> for WitDest {
    fn from(d: PortalDestination) -> Self {
        Self {
            space:  d.space,
            portal: d.portal,
        }
    }
}

fn wit_params_to_shared(p: WitParams) -> PortalParams {
    let t = p.transform;
    PortalParams {
        destination: p.destination.into(),
        size:        [p.size.x, p.size.y],
        transform:   PortalTransform {
            translation: [t.translation.x, t.translation.y, t.translation.z],
            rotation:    [t.rotation.x, t.rotation.y, t.rotation.z, t.rotation.w],
            scale:       [t.scale.x, t.scale.y, t.scale.z],
        },
    }
}

impl bindings::wired::portal::types::Host for Runtime {}

impl HostPortal for Runtime {
    async fn id(&mut self, self_: Resource<PortalRes>) -> wasmtime::Result<String> {
        shared::wired::portal::id(&self.api, self_.rep()).map_err(wasmtime::Error::from_anyhow)
    }

    async fn destination(&mut self, self_: Resource<PortalRes>) -> wasmtime::Result<WitDest> {
        shared::wired::portal::destination(&self.api, self_.rep())
            .map(Into::into)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn close(&mut self, self_: Resource<PortalRes>) -> wasmtime::Result<()> {
        shared::wired::portal::close(&self.api, self_.rep()).map_err(wasmtime::Error::from_anyhow)
    }

    async fn drop(&mut self, rep: Resource<PortalRes>) -> wasmtime::Result<()> {
        shared::wired::portal::on_drop(&self.api, rep.rep()).map_err(wasmtime::Error::from_anyhow)
    }
}

impl bindings::wired::portal::api::Host for Runtime {
    async fn list_portals(&mut self) -> wasmtime::Result<Vec<Resource<ApiPortal>>> {
        shared::wired::portal::list_portals(&self.api)
            .map(|reps| reps.into_iter().map(Resource::new_own).collect())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn open_portal(
        &mut self,
        params: WitParams,
    ) -> wasmtime::Result<Result<Resource<ApiPortal>, String>> {
        Ok(
            shared::wired::portal::open_portal(&self.api, wit_params_to_shared(params))
                .map(Resource::new_own)
                .map_err(|e| e.to_string()),
        )
    }
}
