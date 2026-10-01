//! `wired:physics`: raycasts, and the motion of bodies a script owns.

use std::collections::HashSet;

use avian3d::prelude::{
    AngularVelocity,
    ConstantForce,
    LinearVelocity,
    RigidBody,
    SpatialQuery,
    SpatialQueryFilter,
};
use bevy::{
    ecs::system::SystemState,
    prelude::*,
};
use bevy_hsd::{
    document::{
        DocIndex,
        HsdDocId,
    },
    hierarchy::ancestors,
    prim::{
        Prim,
        PrimIndex,
        PrimOf,
    },
};
use hsd::id::{
    DocId,
    PrimId,
};
use unavi_agent::LocalAgent;
use unavi_physics::finite;
use unavi_policy::permissions::HostApi;

use crate::{
    error::ScriptError,
    host::ScriptHost,
};

/// Most documents one ray filter excludes.
pub const MAX_EXCLUDED_DOCUMENTS: usize = 64;

pub struct RayFilter {
    pub exclude_local_agent: bool,
    pub exclude_documents:   Vec<DocId>,
}

pub struct RayHit {
    pub document: DocId,
    pub prim:     PrimId,
    pub point:    Vec3,
    pub normal:   Vec3,
    pub distance: f32,
}

pub struct BodyVelocity {
    pub linear:  Vec3,
    pub angular: Vec3,
}

/// A guest may pass any bit pattern; every value reaching the solver is
/// checked here first.
fn checked(v: Vec3) -> Result<Vec3, ScriptError> {
    finite::vec3(v.to_array()).ok_or_else(|| ScriptError::invalid("a vector must be finite"))
}

type RayQuery<'w, 's> = (
    SpatialQuery<'w, 's>,
    Query<'w, 's, &'static Prim>,
    Query<'w, 's, &'static PrimOf>,
    Query<'w, 's, &'static HsdDocId>,
    Query<'w, 's, &'static ChildOf>,
    Query<'w, 's, (), With<LocalAgent>>,
);

pub async fn raycast(
    host: &ScriptHost,
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    filter: RayFilter,
) -> Result<Option<RayHit>, ScriptError> {
    host.require(HostApi::Physics)?;
    let origin = checked(origin)?;
    let direction = Dir3::new(checked(direction)?)
        .map_err(|_| ScriptError::invalid("a ray direction must not be zero"))?;
    if !finite::nonneg(max_distance) {
        return Err(ScriptError::invalid(
            "a distance must be finite and not negative",
        ));
    }
    if filter.exclude_documents.len() > MAX_EXCLUDED_DOCUMENTS {
        return Err(ScriptError::invalid(
            "a filter excludes at most 64 documents",
        ));
    }
    let excluded = filter.exclude_documents.into_iter().collect::<HashSet<_>>();

    host.world_call(move |world| {
        let mut state = SystemState::<RayQuery>::new(world);
        let (spatial, prims, prim_of, docs, parents, agents) = state.get(world).ok()?;
        let doc_of = |entity: Entity| {
            ancestors(entity, &parents).find_map(|at| {
                prim_of
                    .get(at)
                    .ok()
                    .and_then(|of| docs.get(of.0).ok())
                    .or_else(|| docs.get(at).ok())
                    .map(|id| id.0)
            })
        };
        let skip = |entity: Entity| {
            (filter.exclude_local_agent
                && ancestors(entity, &parents).any(|at| agents.contains(at)))
                || doc_of(entity).is_some_and(|doc| excluded.contains(&doc))
        };
        let hit = spatial.cast_ray_predicate(
            origin,
            direction,
            max_distance,
            true,
            &SpatialQueryFilter::default(),
            &|entity| !skip(entity),
        )?;
        Some(RayHit {
            document: doc_of(hit.entity)?,
            prim:     prims.get(hit.entity).ok()?.0,
            point:    origin + direction.as_vec3() * hit.distance,
            normal:   hit.normal,
            distance: hit.distance,
        })
    })
    .await
}

fn body(world: &World, doc: DocId, prim: PrimId) -> Result<Entity, ScriptError> {
    let doc = world
        .get_resource::<DocIndex>()
        .and_then(|index| index.get(doc))
        .ok_or(ScriptError::NotFound)?;
    world
        .get::<PrimIndex>(doc)
        .and_then(|prims| prims.get(prim))
        .ok_or(ScriptError::NotReady)
}

/// A body only carries velocity once it is in the simulation, which a
/// document placed this tick is not.
fn dynamic_body(world: &World, doc: DocId, prim: PrimId) -> Result<Entity, ScriptError> {
    let entity = body(world, doc, prim)?;
    match world.get::<RigidBody>(entity) {
        Some(RigidBody::Dynamic) => Ok(entity),
        Some(_) => Err(ScriptError::invalid("not a dynamic body")),
        None => Err(ScriptError::NotReady),
    }
}

pub async fn velocity(
    host: &ScriptHost,
    doc: u32,
    prim: PrimId,
) -> Result<BodyVelocity, ScriptError> {
    host.require(HostApi::Physics)?;
    let doc = host.document(doc)?.id;
    host.world_call(move |world| {
        let entity = dynamic_body(world, doc, prim)?;
        Ok(BodyVelocity {
            linear:  world
                .get::<LinearVelocity>(entity)
                .map_or(Vec3::ZERO, |v| v.0),
            angular: world
                .get::<AngularVelocity>(entity)
                .map_or(Vec3::ZERO, |v| v.0),
        })
    })
    .await?
}

pub async fn set_velocity(
    host: &ScriptHost,
    doc: u32,
    prim: PrimId,
    linear: Option<Vec3>,
    angular: Option<Vec3>,
) -> Result<(), ScriptError> {
    host.require(HostApi::Physics)?;
    let doc = host.owned_document(doc)?.id;
    let linear = linear.map(checked).transpose()?;
    let angular = angular.map(checked).transpose()?;
    host.world_call(move |world| {
        let entity = dynamic_body(world, doc, prim)?;
        let mut body = world.entity_mut(entity);
        if let Some(linear) = linear {
            body.insert(LinearVelocity(linear));
        }
        if let Some(angular) = angular {
            body.insert(AngularVelocity(angular));
        }
        Ok(())
    })
    .await?
}

/// A force the solver applies every step until set again; zero clears it.
pub async fn set_force(
    host: &ScriptHost,
    doc: u32,
    prim: PrimId,
    force: Vec3,
) -> Result<(), ScriptError> {
    host.require(HostApi::Physics)?;
    let doc = host.owned_document(doc)?.id;
    let force = checked(force)?;
    host.world_call(move |world| {
        let entity = dynamic_body(world, doc, prim)?;
        let mut body = world.entity_mut(entity);
        if force == Vec3::ZERO {
            body.remove::<ConstantForce>();
        } else {
            body.insert(ConstantForce(force));
        }
        Ok(())
    })
    .await?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_non_finite_vector_is_refused() {
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(checked(Vec3::new(0.0, bad, 0.0)).is_err());
        }
        assert_eq!(checked(Vec3::ONE), Ok(Vec3::ONE));
    }
}
