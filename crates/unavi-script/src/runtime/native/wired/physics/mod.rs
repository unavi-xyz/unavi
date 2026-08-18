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
        path: "../../protocol/wit/wired-physics",
        with: {
            "wired:scene/types.prim": PrimRes,
            "wired:error/types": crate::runtime::native::wired::error::bindings::wired::error::types,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

use bindings::wired::physics::types::RayHit;

use crate::runtime::native::wired::error::bindings::wired::error::types::Error;

impl bindings::wired::physics::types::Host for Runtime {}

impl bindings::wired::physics::api::Host for Runtime {
    async fn raycast(
        &mut self,
        origin: bindings::wired::math::types::Vec3,
        dir: bindings::wired::math::types::Vec3,
        max_dist: f32,
    ) -> wasmtime::Result<Result<Option<RayHit>, Error>> {
        if let Err(err) = self.api.require(ApiName::Physics) {
            return Ok(Err(err.into()));
        }
        let result = shared::wired::physics::raycast(
            &self.api,
            [origin.x, origin.y, origin.z],
            [dir.x, dir.y, dir.z],
            max_dist,
        )
        .await;
        Ok(result.map(|hit| hit.map(into_wit_hit)).map_err(Into::into))
    }

    async fn get_linear_velocity(
        &mut self,
        prim: Resource<PrimRes>,
    ) -> wasmtime::Result<Result<bindings::wired::math::types::Vec3, Error>> {
        let result = match self.api.require(ApiName::Physics) {
            Ok(()) => shared::wired::physics::get_linear_velocity(&self.api, prim.rep()).await,
            Err(err) => Err(err),
        };
        Ok(result.map(vec3).map_err(Into::into))
    }

    async fn set_linear_velocity(
        &mut self,
        prim: Resource<PrimRes>,
        v: bindings::wired::math::types::Vec3,
    ) -> wasmtime::Result<Result<(), Error>> {
        let result = match self.api.require(ApiName::Physics) {
            Ok(()) => {
                shared::wired::physics::set_linear_velocity(&self.api, prim.rep(), [v.x, v.y, v.z])
                    .await
            }
            Err(err) => Err(err),
        };
        Ok(result.map_err(Into::into))
    }

    async fn apply_force(
        &mut self,
        prim: Resource<PrimRes>,
        force: bindings::wired::math::types::Vec3,
    ) -> wasmtime::Result<Result<(), Error>> {
        let result = match self.api.require(ApiName::Physics) {
            Ok(()) => {
                shared::wired::physics::apply_force(
                    &self.api,
                    prim.rep(),
                    [force.x, force.y, force.z],
                )
                .await
            }
            Err(err) => Err(err),
        };
        Ok(result.map_err(Into::into))
    }

    async fn set_angular_velocity(
        &mut self,
        prim: Resource<PrimRes>,
        v: bindings::wired::math::types::Vec3,
    ) -> wasmtime::Result<Result<(), Error>> {
        let result = match self.api.require(ApiName::Physics) {
            Ok(()) => {
                shared::wired::physics::set_angular_velocity(&self.api, prim.rep(), [v.x, v.y, v.z])
                    .await
            }
            Err(err) => Err(err),
        };
        Ok(result.map_err(Into::into))
    }

    async fn claim_authority(&mut self, doc: Vec<u8>) -> wasmtime::Result<Result<(), Error>> {
        let result = match self.api.require(ApiName::Physics) {
            Ok(()) => shared::wired::physics::claim_authority(&self.api, doc),
            Err(err) => Err(err),
        };
        Ok(result.map_err(Into::into))
    }

    async fn release_authority(&mut self, doc: Vec<u8>) -> wasmtime::Result<Result<(), Error>> {
        let result = match self.api.require(ApiName::Physics) {
            Ok(()) => shared::wired::physics::release_authority(&self.api, doc),
            Err(err) => Err(err),
        };
        Ok(result.map_err(Into::into))
    }
}

fn into_wit_hit(hit: shared::wired::physics::RayHit) -> RayHit {
    RayHit {
        document: hit.document,
        prim:     hit.prim,
        point:    vec3(hit.point),
        normal:   vec3(hit.normal),
        distance: hit.distance,
    }
}

const fn vec3(v: [f32; 3]) -> bindings::wired::math::types::Vec3 {
    bindings::wired::math::types::Vec3 {
        x: v[0],
        y: v[1],
        z: v[2],
    }
}
