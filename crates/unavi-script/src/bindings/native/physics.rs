//! `wired:physics`.

use wasmtime::component::Resource;

use super::{
    HostCtx,
    convert,
    generated::wired::{
        core::{
            ids::PrimId,
            math::{
                Ray,
                Vec3,
            },
        },
        physics::simulation::{
            BodyVelocity,
            Host,
            RayFilter,
            RayHit,
        },
    },
};
use crate::{
    error::ScriptError,
    host::{
        physics,
        scene::DocumentRes,
    },
};

impl Host for HostCtx {
    async fn raycast(
        &mut self,
        ray: Ray,
        max_distance: f32,
        filter: RayFilter,
    ) -> Result<Option<RayHit>, ScriptError> {
        if filter.exclude_documents.len() > physics::MAX_EXCLUDED_DOCUMENTS {
            return Err(ScriptError::invalid(
                "a filter excludes at most 64 documents",
            ));
        }
        let (origin, direction) = convert::ray(ray);
        let filter = physics::RayFilter {
            exclude_local_agent: filter.exclude_local_agent,
            exclude_documents:   filter
                .exclude_documents
                .into_iter()
                .map(convert::doc_id)
                .collect(),
        };
        Ok(
            physics::raycast(&self.host, origin, direction, max_distance, filter)
                .await?
                .map(|hit| RayHit {
                    target:   convert::wit_prim_ref((hit.document, hit.prim)),
                    point:    convert::wit_vec3(hit.point),
                    normal:   convert::wit_vec3(hit.normal),
                    distance: hit.distance,
                }),
        )
    }

    async fn velocity(
        &mut self,
        doc: Resource<DocumentRes>,
        prim: PrimId,
    ) -> Result<BodyVelocity, ScriptError> {
        let velocity = physics::velocity(&self.host, doc.rep(), convert::prim_id(prim)).await?;
        Ok(BodyVelocity {
            linear:  convert::wit_vec3(velocity.linear),
            angular: convert::wit_vec3(velocity.angular),
        })
    }

    async fn set_velocity(
        &mut self,
        doc: Resource<DocumentRes>,
        prim: PrimId,
        linear: Option<Vec3>,
        angular: Option<Vec3>,
    ) -> Result<(), ScriptError> {
        physics::set_velocity(
            &self.host,
            doc.rep(),
            convert::prim_id(prim),
            linear.map(convert::vec3),
            angular.map(convert::vec3),
        )
        .await
    }

    async fn set_force(
        &mut self,
        doc: Resource<DocumentRes>,
        prim: PrimId,
        force: Vec3,
    ) -> Result<(), ScriptError> {
        physics::set_force(
            &self.host,
            doc.rep(),
            convert::prim_id(prim),
            convert::vec3(force),
        )
        .await
    }
}
