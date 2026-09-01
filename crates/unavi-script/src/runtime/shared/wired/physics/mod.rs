use avian3d::prelude::{
    AngularVelocity,
    ConstantForce,
    LinearVelocity,
    RigidBody,
    SpatialQuery,
    SpatialQueryFilter,
};
use bevy::{
    ecs::system::RunSystemOnce,
    prelude::*,
};
use bevy_hsd::{
    HsdChild,
    HsdDocId,
    HsdPrimIndex,
    Prim as HsdPrim,
};
use hsd::id::{
    DocId,
    PrimId,
};
use unavi_physics::finite;
use unavi_util::{
    async_commands::AsyncCommands,
    hierarchy::ancestors,
};

use crate::{
    error::ScriptError,
    runtime::shared::Api,
};

/// A guest may pass any bit pattern. Every value crossing into avian is checked
/// here, before it can reach a component the solver reads.
fn checked_vec3(what: &str, v: [f32; 3]) -> Result<Vec3, ScriptError> {
    finite::vec3(v).ok_or_else(|| ScriptError::other(format!("{what} must be finite, got {v:?}")))
}

fn checked_distance(what: &str, v: f32) -> Result<f32, ScriptError> {
    if finite::nonneg(v) {
        Ok(v)
    } else {
        Err(ScriptError::other(format!(
            "{what} must be finite and >= 0, got {v}"
        )))
    }
}

pub struct RayHit {
    pub document: Vec<u8>,
    pub prim:     String,
    pub point:    [f32; 3],
    pub normal:   [f32; 3],
    pub distance: f32,
}

fn resolve_doc(
    entity: Entity,
    children: &Query<&HsdChild>,
    docs: &Query<&HsdDocId>,
    parents: &Query<&ChildOf>,
) -> Option<DocId> {
    ancestors(entity, parents).find_map(|at| {
        children
            .get(at)
            .ok()
            .and_then(|child| docs.get(child.0).ok())
            .or_else(|| docs.get(at).ok())
            .map(|rec| rec.0)
    })
}

pub async fn raycast(
    _api: &Api,
    origin: [f32; 3],
    dir: [f32; 3],
    max_dist: f32,
) -> Result<Option<RayHit>, ScriptError> {
    let origin_v = checked_vec3("raycast origin", origin)?;
    let dir_v = checked_vec3("raycast direction", dir)?;
    let max_dist = checked_distance("raycast distance", max_dist)?;

    let (tx, rx) = async_channel::bounded::<Option<RayHit>>(1);
    AsyncCommands::default()
        .push(move |world: &mut World| {
            let hit = world
                .run_system_once(
                    move |spatial: SpatialQuery,
                          prims: Query<&HsdPrim>,
                          children: Query<&HsdChild>,
                          docs: Query<&HsdDocId>,
                          parents: Query<&ChildOf>|
                          -> Option<RayHit> {
                        let direction = Dir3::new(dir_v).ok()?;
                        let hit = spatial.cast_ray(
                            origin_v,
                            direction,
                            max_dist,
                            true,
                            &SpatialQueryFilter::default(),
                        )?;
                        let document = resolve_doc(hit.entity, &children, &docs, &parents)?;
                        let hit_prim = prims.get(hit.entity).ok()?.0;
                        let point = origin_v + direction.as_vec3() * hit.distance;
                        Some(RayHit {
                            document: document.0.to_vec(),
                            prim:     hit_prim.to_string(),
                            point:    point.to_array(),
                            normal:   hit.normal.to_array(),
                            distance: hit.distance,
                        })
                    },
                )
                .ok()
                .flatten();
            tx.try_send(hit).ok();
        })
        .send()
        .await
        .map_err(|err| ScriptError::other(err.to_string()))?;
    rx.recv()
        .await
        .map_err(|err| ScriptError::other(err.to_string()))
}

async fn prim_ident(api: &Api, prim_rep: u32) -> Result<(DocId, PrimId), ScriptError> {
    let scene = api.wired_scene.lock().await;
    let prim = scene
        .prims
        .get(prim_rep)
        .ok_or_else(|| ScriptError::other(format!("invalid prim rep: {prim_rep}")))?;
    let ident = (prim.doc_id, prim.id);
    drop(scene);
    Ok(ident)
}

fn entity_for(world: &mut World, doc: DocId, prim: PrimId) -> Option<Entity> {
    let mut query = world.query::<(&HsdDocId, &HsdPrimIndex)>();
    for (rec, index) in query.iter(world) {
        if rec.0 == doc {
            return index.0.get(&prim).copied();
        }
    }
    None
}

async fn set_velocity(
    api: &Api,
    prim_rep: u32,
    v: [f32; 3],
    angular: bool,
) -> Result<(), ScriptError> {
    let vel = checked_vec3(
        if angular {
            "angular velocity"
        } else {
            "linear velocity"
        },
        v,
    )?;
    let (doc, prim_id) = prim_ident(api, prim_rep).await?;
    let (tx, rx) = async_channel::bounded(1);
    AsyncCommands::default()
        .push(move |world: &mut World| {
            tx.try_send(apply_velocity(world, doc, prim_id, vel, angular))
                .ok();
        })
        .send()
        .await
        .map_err(|err| ScriptError::other(err.to_string()))?;
    rx.recv()
        .await
        .map_err(|err| ScriptError::other(err.to_string()))?
}

/// A body only carries velocity once it is realized, which a document placed
/// this tick is not; reported rather than dropped, so a caller can try again
/// instead of quietly landing a thrown thing on the spot.
fn apply_velocity(
    world: &mut World,
    doc: DocId,
    prim: PrimId,
    vel: Vec3,
    angular: bool,
) -> Result<(), ScriptError> {
    let Some(entity) = entity_for(world, doc, prim) else {
        return Err(ScriptError::other(format!(
            "prim {prim} of {doc} has no body in the world"
        )));
    };
    if !matches!(world.get::<RigidBody>(entity), Some(RigidBody::Dynamic)) {
        return Err(ScriptError::other(format!(
            "prim {prim} of {doc} is not a dynamic body"
        )));
    }
    let mut ent = world.entity_mut(entity);
    if angular {
        ent.insert(AngularVelocity(vel));
    } else {
        ent.insert(LinearVelocity(vel));
    }
    Ok(())
}

pub async fn set_linear_velocity(api: &Api, prim_rep: u32, v: [f32; 3]) -> Result<(), ScriptError> {
    set_velocity(api, prim_rep, v, false).await
}

pub async fn set_angular_velocity(
    api: &Api,
    prim_rep: u32,
    v: [f32; 3],
) -> Result<(), ScriptError> {
    set_velocity(api, prim_rep, v, true).await
}

pub async fn get_linear_velocity(api: &Api, prim_rep: u32) -> Result<[f32; 3], ScriptError> {
    let (doc, prim_id) = prim_ident(api, prim_rep).await?;
    let (tx, rx) = async_channel::bounded::<[f32; 3]>(1);
    AsyncCommands::default()
        .push(move |world: &mut World| {
            let v = entity_for(world, doc, prim_id)
                .and_then(|entity| world.get::<LinearVelocity>(entity))
                .map_or([0.0; 3], |lv| lv.0.to_array());
            tx.try_send(v).ok();
        })
        .send()
        .await
        .map_err(|err| ScriptError::other(err.to_string()))?;
    rx.recv()
        .await
        .map_err(|err| ScriptError::other(err.to_string()))
}

/// Sets a persistent world-space force (avian `ConstantForce`); the solver
/// reads it every step until changed. A zero vector removes it.
pub async fn apply_force(api: &Api, prim_rep: u32, v: [f32; 3]) -> Result<(), ScriptError> {
    let value = checked_vec3("force", v)?;
    let (doc, prim_id) = prim_ident(api, prim_rep).await?;
    AsyncCommands::default()
        .push(move |world: &mut World| {
            let Some(entity) = entity_for(world, doc, prim_id) else {
                return;
            };
            if !matches!(world.get::<RigidBody>(entity), Some(RigidBody::Dynamic)) {
                return;
            }
            let mut ent = world.entity_mut(entity);
            if value == Vec3::ZERO {
                ent.remove::<ConstantForce>();
            } else {
                ent.insert(ConstantForce(value));
            }
        })
        .send()
        .await
        .map_err(|err| ScriptError::other(err.to_string()))?;
    Ok(())
}

pub fn claim_authority(api: &Api, doc_id: Vec<u8>) -> Result<(), ScriptError> {
    let bytes = <[u8; 32]>::try_from(doc_id.as_slice())
        .map_err(|_| ScriptError::other("document id must be 32 bytes"))?;
    let doc = DocId(bytes);
    let space = api
        .view
        .space_of(doc)
        .ok_or_else(|| ScriptError::other("document is not in a tracked space"))?;
    api.view.claim_authority(space, doc);
    Ok(())
}

pub fn release_authority(_api: &Api, doc_id: Vec<u8>) -> Result<(), ScriptError> {
    let bytes = <[u8; 32]>::try_from(doc_id.as_slice())
        .map_err(|_| ScriptError::other("document id must be 32 bytes"))?;
    unavi_space::state::entities::release_authority(DocId(bytes));
    Ok(())
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Vec3;

    use super::{
        checked_distance,
        checked_vec3,
    };

    #[test]
    fn a_finite_vector_is_accepted() {
        assert_eq!(
            checked_vec3("force", [1.0, -2.0, 3.0]).expect("finite vector rejected"),
            Vec3::new(1.0, -2.0, 3.0)
        );
    }

    #[test]
    fn a_non_finite_vector_is_refused_and_names_the_parameter() {
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let err = checked_vec3("linear velocity", [0.0, bad, 0.0])
                .expect_err(&format!("{bad} was accepted"));
            assert!(
                err.to_string().contains("linear velocity"),
                "error does not say which parameter was bad: {err}"
            );
        }
    }

    #[test]
    fn a_negative_or_non_finite_distance_is_refused() {
        assert!(checked_distance("raycast distance", 0.0).is_ok());
        assert!(checked_distance("raycast distance", 10.0).is_ok());
        assert!(checked_distance("raycast distance", -1.0).is_err());
        assert!(checked_distance("raycast distance", f32::NAN).is_err());
        assert!(checked_distance("raycast distance", f32::INFINITY).is_err());
    }
}
