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
    HsdNamespace,
    HsdPrimIndex,
    Prim as HsdPrim,
};
use iroh_docs::NamespaceId;
use loro::TreeID;
use unavi_util::async_commands::AsyncCommands;

use crate::{
    error::ScriptError,
    runtime::shared::Api,
};

pub struct RayHit {
    pub document: Vec<u8>,
    pub prim:     String,
    pub point:    [f32; 3],
    pub normal:   [f32; 3],
    pub distance: f32,
}

fn resolve_doc(
    mut entity: Entity,
    children: &Query<&HsdChild>,
    docs: &Query<&HsdNamespace>,
    parents: &Query<&ChildOf>,
) -> Option<NamespaceId> {
    loop {
        if let Ok(child) = children.get(entity)
            && let Ok(rec) = docs.get(child.0)
        {
            return Some(rec.0);
        }
        if let Ok(rec) = docs.get(entity) {
            return Some(rec.0);
        }
        match parents.get(entity) {
            Ok(parent) => entity = parent.parent(),
            Err(_) => return None,
        }
    }
}

pub async fn raycast(
    _api: &Api,
    origin: [f32; 3],
    dir: [f32; 3],
    max_dist: f32,
) -> Result<Option<RayHit>, ScriptError> {
    let (tx, rx) = async_channel::bounded::<Option<RayHit>>(1);
    AsyncCommands::default()
        .push(move |world: &mut World| {
            let hit = world
                .run_system_once(
                    move |spatial: SpatialQuery,
                          prims: Query<&HsdPrim>,
                          children: Query<&HsdChild>,
                          docs: Query<&HsdNamespace>,
                          parents: Query<&ChildOf>|
                          -> Option<RayHit> {
                        let origin_v = Vec3::from_array(origin);
                        let direction = Dir3::new(Vec3::from_array(dir)).ok()?;
                        let hit = spatial.cast_ray(
                            origin_v,
                            direction,
                            max_dist,
                            true,
                            &SpatialQueryFilter::default(),
                        )?;
                        let document = resolve_doc(hit.entity, &children, &docs, &parents)?;
                        let tree = prims.get(hit.entity).ok()?.0;
                        let point = origin_v + direction.as_vec3() * hit.distance;
                        Some(RayHit {
                            document: document.as_bytes().to_vec(),
                            prim:     tree.to_string(),
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

async fn prim_ident(api: &Api, prim_rep: u32) -> Result<(NamespaceId, TreeID), ScriptError> {
    let scene = api.wired_scene.lock().await;
    let prim = scene
        .prims
        .get(prim_rep)
        .ok_or_else(|| ScriptError::other(format!("invalid prim rep: {prim_rep}")))?;
    let ident = (prim.doc_id, prim.id);
    drop(scene);
    Ok(ident)
}

fn entity_for(world: &mut World, doc: NamespaceId, tree: TreeID) -> Option<Entity> {
    let mut query = world.query::<(&HsdNamespace, &HsdPrimIndex)>();
    for (rec, index) in query.iter(world) {
        if rec.0 == doc {
            return index.0.get(&tree).copied();
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
    let (doc, tree) = prim_ident(api, prim_rep).await?;
    AsyncCommands::default()
        .push(move |world: &mut World| {
            let Some(entity) = entity_for(world, doc, tree) else {
                return;
            };
            if !matches!(world.get::<RigidBody>(entity), Some(RigidBody::Dynamic)) {
                return;
            }
            let vel = Vec3::from_array(v);
            let mut ent = world.entity_mut(entity);
            if angular {
                ent.insert(AngularVelocity(vel));
            } else {
                ent.insert(LinearVelocity(vel));
            }
        })
        .send()
        .await
        .map_err(|err| ScriptError::other(err.to_string()))?;
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
    let (doc, tree) = prim_ident(api, prim_rep).await?;
    let (tx, rx) = async_channel::bounded::<[f32; 3]>(1);
    AsyncCommands::default()
        .push(move |world: &mut World| {
            let v = entity_for(world, doc, tree)
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
/// reads it every step until changed. A zero vector removes it, so a hold that
/// stops refreshing must pass zero to stop pushing.
pub async fn apply_force(api: &Api, prim_rep: u32, v: [f32; 3]) -> Result<(), ScriptError> {
    let (doc, tree) = prim_ident(api, prim_rep).await?;
    AsyncCommands::default()
        .push(move |world: &mut World| {
            let Some(entity) = entity_for(world, doc, tree) else {
                return;
            };
            if !matches!(world.get::<RigidBody>(entity), Some(RigidBody::Dynamic)) {
                return;
            }
            let value = Vec3::from_array(v);
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

pub fn claim_authority(_api: &Api, doc_id: Vec<u8>) -> Result<(), ScriptError> {
    let bytes = <[u8; 32]>::try_from(doc_id.as_slice())
        .map_err(|_| ScriptError::other("document id must be 32 bytes"))?;
    let doc = NamespaceId::from(&bytes);
    let space = unavi_space::membership::doc_space(doc)
        .ok_or_else(|| ScriptError::other("document is not in a tracked space"))?;
    unavi_space::state::entities::claim_authority(space, doc);
    Ok(())
}

pub fn release_authority(_api: &Api, doc_id: Vec<u8>) -> Result<(), ScriptError> {
    let bytes = <[u8; 32]>::try_from(doc_id.as_slice())
        .map_err(|_| ScriptError::other("document id must be 32 bytes"))?;
    unavi_space::state::entities::release_authority(NamespaceId::from(&bytes));
    Ok(())
}
