use avian3d::prelude::{
    AngularDamping, AngularInertia, Collider, ComputedAngularInertia, LinearDamping, RigidBody,
    Sensor,
};
use bevy::prelude::*;
use loro::TreeID;
use smol_str::SmolStr;

use hsd::{HsdCollider, HsdNode, HsdRigidBody};

use crate::{
    HsdChild, HsdEntityMaps, HsdNodePhysics, HsdRecordId, HsdScript, MaterialId, MeshId, NodeId,
    NodeScripts, ScriptNode,
    hydrate::{
        compile::{collider::ColliderParams, material::CompiledMaterial, mesh::CompiledMesh},
        events::NodeRef,
    },
};

use super::collider::insert_collider;

#[derive(Event)]
pub struct HsdDocTransformSet {
    pub doc_id: blake3::Hash,
    pub transform: Transform,
}

pub(crate) fn handle_hsd_doc_transform_set(
    trigger: On<HsdDocTransformSet>,
    docs: Query<(Entity, &HsdRecordId)>,
    mut transforms: Query<&mut Transform, With<crate::HsdDoc>>,
) {
    let ev = trigger.event();
    let Some((doc_ent, _)) = docs.iter().find(|(_, id)| id.0 == ev.doc_id) else {
        return;
    };
    if let Ok(mut t) = transforms.get_mut(doc_ent) {
        *t = ev.transform;
    }
}

#[derive(Component)]
pub struct MeshRef(pub SmolStr);

#[derive(Component)]
pub struct MaterialRef(pub SmolStr);

#[derive(Event)]
pub struct HsdNodeColliderSet {
    pub doc_id: blake3::Hash,
    pub id: TreeID,
    pub collider: Option<HsdCollider>,
}

#[derive(Event)]
pub struct HsdNodeDespawned {
    pub doc_id: blake3::Hash,
    pub id: TreeID,
}

#[derive(Event)]
pub struct HsdNodeMaterialSet {
    pub doc_id: blake3::Hash,
    pub id: TreeID,
    pub material: Option<SmolStr>,
}

#[derive(Event)]
pub struct HsdNodeMeshSet {
    pub doc_id: blake3::Hash,
    pub id: TreeID,
    pub mesh: Option<SmolStr>,
}

#[derive(Event)]
pub struct HsdNodeNameSet {
    pub doc_id: blake3::Hash,
    pub id: TreeID,
    pub name: Option<String>,
}

#[derive(Event)]
pub struct HsdNodeParentSet {
    pub doc_id: blake3::Hash,
    pub child: NodeRef,
    pub parent: Option<NodeRef>,
}

#[derive(Event)]
pub struct HsdNodeRigidBodySet {
    pub doc_id: blake3::Hash,
    pub id: TreeID,
    pub rigid_body: Option<HsdRigidBody>,
}

#[derive(Event)]
pub struct HsdNodeScriptsSet {
    pub doc_id: blake3::Hash,
    pub id: TreeID,
    pub scripts: Vec<blake3::Hash>,
}

#[derive(Event)]
pub struct HsdNodeSpawned {
    pub doc_id: blake3::Hash,
    pub id: TreeID,
}

#[derive(Event)]
pub struct HsdNodeTransformSet {
    pub doc_id: blake3::Hash,
    pub id: TreeID,
    pub transform: Transform,
}

pub(crate) fn handle_hsd_node_spawned(
    trigger: On<HsdNodeSpawned>,
    docs: Query<(Entity, &HsdRecordId)>,
    mut entity_maps: Query<&mut HsdEntityMaps>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, "node spawned");
    let Some((doc_ent, _)) = docs.iter().find(|(_, id)| id.0 == ev.doc_id) else {
        return;
    };
    let Ok(mut maps) = entity_maps.get_mut(doc_ent) else {
        return;
    };
    if maps.nodes.contains_key(&ev.id) {
        return;
    }
    let ent = commands
        .spawn((
            ChildOf(doc_ent),
            HsdChild(doc_ent),
            NodeId(ev.id),
            Transform::default(),
            Visibility::default(),
        ))
        .id();
    maps.nodes.insert(ev.id, ent);
}

pub(crate) fn handle_hsd_node_despawned(
    trigger: On<HsdNodeDespawned>,
    docs: Query<(Entity, &HsdRecordId)>,
    mut entity_maps: Query<&mut HsdEntityMaps>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, "node despawned");
    let Some((doc_ent, _)) = docs.iter().find(|(_, id)| id.0 == ev.doc_id) else {
        return;
    };
    let Ok(mut maps) = entity_maps.get_mut(doc_ent) else {
        return;
    };
    let Some(ent) = maps.nodes.remove(&ev.id) else {
        return;
    };
    if let Ok(mut entity_cmd) = commands.get_entity(ent) {
        entity_cmd.despawn();
    }
}

pub(crate) fn handle_hsd_node_collider_set(
    trigger: On<HsdNodeColliderSet>,
    docs: Query<(Entity, &HsdRecordId)>,
    entity_maps: Query<&HsdEntityMaps>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, has_collider = ev.collider.is_some(), "node collider set");
    let Some(ent) = node_entity(&docs, &entity_maps, &ev.doc_id, &ev.id) else {
        return;
    };
    let Ok(mut entity_cmd) = commands.get_entity(ent) else {
        return;
    };
    let collider = ev.collider.clone();
    entity_cmd
        .entry::<HsdNodePhysics>()
        .or_default()
        .and_modify(move |mut p| p.collider = collider);
    let Ok(mut entity_cmd) = commands.get_entity(ent) else {
        return;
    };
    entity_cmd
        .try_remove::<Collider>()
        .try_remove::<ColliderParams>();
}

pub(crate) fn handle_hsd_node_material_set(
    trigger: On<HsdNodeMaterialSet>,
    docs: Query<(Entity, &HsdRecordId)>,
    entity_maps: Query<&HsdEntityMaps>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, material = ?ev.material, "node material set");
    let Some(ent) = node_entity(&docs, &entity_maps, &ev.doc_id, &ev.id) else {
        return;
    };
    let Ok(mut entity_cmd) = commands.get_entity(ent) else {
        return;
    };
    entity_cmd.try_remove::<MaterialRef>();
    if let Some(ref id) = ev.material {
        entity_cmd.insert(MaterialRef(id.clone()));
    }
}

pub(crate) fn handle_hsd_node_mesh_set(
    trigger: On<HsdNodeMeshSet>,
    docs: Query<(Entity, &HsdRecordId)>,
    entity_maps: Query<&HsdEntityMaps>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, mesh = ?ev.mesh, "node mesh set");
    let Some(ent) = node_entity(&docs, &entity_maps, &ev.doc_id, &ev.id) else {
        return;
    };
    let Ok(mut entity_cmd) = commands.get_entity(ent) else {
        return;
    };
    entity_cmd.try_remove::<MeshRef>();
    if let Some(ref id) = ev.mesh {
        entity_cmd.insert(MeshRef(id.clone()));
    }
}

pub(crate) fn handle_hsd_node_name_set(
    trigger: On<HsdNodeNameSet>,
    docs: Query<(Entity, &HsdRecordId)>,
    entity_maps: Query<&HsdEntityMaps>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, name = ?ev.name, "node name set");
    let Some(ent) = node_entity(&docs, &entity_maps, &ev.doc_id, &ev.id) else {
        return;
    };
    let Ok(mut entity_cmd) = commands.get_entity(ent) else {
        return;
    };
    if let Some(ref name) = ev.name {
        entity_cmd.insert(Name::new(name.clone()));
    } else {
        entity_cmd.try_remove::<Name>();
    }
}

pub(crate) fn handle_hsd_node_parent_set(
    trigger: On<HsdNodeParentSet>,
    docs: Query<(Entity, &HsdRecordId)>,
    entity_maps: Query<&HsdEntityMaps>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(child = ?ev.child, parent = ?ev.parent, "node parent set");
    let Some((doc_ent, _)) = docs.iter().find(|(_, id)| id.0 == ev.doc_id) else {
        return;
    };
    let Ok(maps) = entity_maps.get(doc_ent) else {
        return;
    };

    let child_ent = match &ev.child {
        NodeRef::Entity(e) => Some(*e),
        NodeRef::Id(id) => maps.nodes.get(id).copied(),
    };
    let Some(child_ent) = child_ent else { return };

    let parent_ent = match &ev.parent {
        None => doc_ent,
        Some(NodeRef::Entity(p)) => *p,
        Some(NodeRef::Id(pid)) => {
            let Some(&ent) = maps.nodes.get(pid) else {
                return;
            };
            ent
        }
    };

    if let Ok(mut ent) = commands.get_entity(child_ent) {
        ent.insert(ChildOf(parent_ent));
    }
}

const DAMPING_DEFAULT: f32 = 0.2;

pub(crate) fn insert_rigid_body(ent: Entity, data: &HsdRigidBody, commands: &mut Commands) {
    let kind = match data.kind.as_str() {
        "dynamic" => RigidBody::Dynamic,
        // Static x static collisions panic avian.
        "fixed" | "kinematic" => RigidBody::Kinematic,
        other => {
            warn!("invalid rigid body kind: {other}");
            RigidBody::default()
        }
    };

    let mut entity_cmd = commands.entity(ent);
    entity_cmd.insert(kind);

    if kind == RigidBody::Dynamic {
        let linear = data.linear_damping.map_or(DAMPING_DEFAULT, |v| v as f32);
        let angular = data.angular_damping.map_or(DAMPING_DEFAULT, |v| v as f32);
        entity_cmd.insert((LinearDamping(linear), AngularDamping(angular)));
    }
}

pub(crate) fn handle_hsd_node_rigid_body_set(
    trigger: On<HsdNodeRigidBodySet>,
    docs: Query<(Entity, &HsdRecordId)>,
    entity_maps: Query<&HsdEntityMaps>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, kind = ?ev.rigid_body.as_ref().map(|r| &r.kind), "node rigid body set");
    let Some(ent) = node_entity(&docs, &entity_maps, &ev.doc_id, &ev.id) else {
        return;
    };
    let Ok(mut entity_cmd) = commands.get_entity(ent) else {
        return;
    };
    let rigid_body = ev.rigid_body.clone();
    entity_cmd
        .entry::<HsdNodePhysics>()
        .or_default()
        .and_modify(move |mut p| p.rigid_body = rigid_body);
    let Ok(mut entity_cmd) = commands.get_entity(ent) else {
        return;
    };
    entity_cmd.try_remove::<RigidBody>();
}

pub(crate) fn handle_hsd_node_scripts_set(
    trigger: On<HsdNodeScriptsSet>,
    docs: Query<(Entity, &HsdRecordId)>,
    entity_maps: Query<&HsdEntityMaps>,
    names: Query<NameOrEntity>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, count = ev.scripts.len(), "node scripts set");
    let Some(entity) = node_entity(&docs, &entity_maps, &ev.doc_id, &ev.id) else {
        return;
    };
    // TODO handle removed scripts
    if ev.scripts.is_empty() {
        let Ok(mut entity_cmd) = commands.get_entity(entity) else {
            return;
        };
        entity_cmd.try_remove::<NodeScripts>();
    } else {
        let name_base = names.get(entity).expect("name");

        for (i, hash) in ev.scripts.iter().enumerate() {
            let name = if ev.scripts.len() == 1 {
                Name::new(name_base.to_string())
            } else {
                Name::new(format!("{name_base}.{i}"))
            };
            commands.spawn((ScriptNode(entity), HsdScript(*hash), name));
        }
    }
}

pub(crate) fn handle_hsd_node_transform_set(
    trigger: On<HsdNodeTransformSet>,
    docs: Query<(Entity, &HsdRecordId)>,
    entity_maps: Query<&HsdEntityMaps>,
    mut transforms: Query<&mut Transform>,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, "node transform set");
    let Some(ent) = node_entity(&docs, &entity_maps, &ev.doc_id, &ev.id) else {
        return;
    };
    if let Ok(mut t) = transforms.get_mut(ent) {
        *t = ev.transform;
    }
}

pub(crate) fn on_mesh_ref_set(
    trigger: On<Add, MeshRef>,
    nodes: Query<(&MeshRef, &HsdChild), With<NodeId>>,
    entity_maps: Query<&HsdEntityMaps>,
    compiled_meshes: Query<&CompiledMesh>,
    mut commands: Commands,
) {
    let node_ent = trigger.entity;
    let Ok((mesh_ref, mesh_doc)) = nodes.get(node_ent) else {
        return;
    };
    let Ok(maps) = entity_maps.get(mesh_doc.0) else {
        return;
    };
    let Some(&mesh_ent) = maps.meshes.get(&mesh_ref.0) else {
        return;
    };
    let Ok(compiled_mesh) = compiled_meshes.get(mesh_ent) else {
        return;
    };
    if let Ok(mut entity_cmd) = commands.get_entity(node_ent) {
        entity_cmd.insert(Mesh3d(compiled_mesh.0.clone()));
    }
}

pub(crate) fn on_material_ref_set(
    trigger: On<Add, MaterialRef>,
    nodes: Query<(&MaterialRef, &HsdChild), With<NodeId>>,
    entity_maps: Query<&HsdEntityMaps>,
    compiled_mats: Query<&CompiledMaterial>,
    mut commands: Commands,
) {
    let node_ent = trigger.entity;
    let Ok((mat_ref, hsd_child)) = nodes.get(node_ent) else {
        return;
    };
    let Ok(maps) = entity_maps.get(hsd_child.0) else {
        return;
    };
    let Some(&mat_ent) = maps.materials.get(&mat_ref.0) else {
        return;
    };
    let Ok(compiled_mat) = compiled_mats.get(mat_ent) else {
        return;
    };
    if let Ok(mut entity_cmd) = commands.get_entity(node_ent) {
        entity_cmd.insert(MeshMaterial3d(compiled_mat.0.clone()));
    }
}

pub(crate) fn on_mesh_ref_removed(trigger: On<Remove, MeshRef>, mut commands: Commands) {
    if let Ok(mut entity_cmd) = commands.get_entity(trigger.entity) {
        entity_cmd.try_remove::<Mesh3d>();
    }
}

pub(crate) fn on_mesh_compiled(
    trigger: On<Add, CompiledMesh>,
    mesh_query: Query<(&HsdChild, &CompiledMesh, &MeshId)>,
    node_refs: Query<(Entity, &MeshRef, &HsdChild), With<NodeId>>,
    entity_maps: Query<&HsdEntityMaps>,
    compiled_mats: Query<&CompiledMaterial>,
    mat_refs: Query<&MaterialRef>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut default_material: Local<Option<Handle<StandardMaterial>>>,
) {
    let mesh_ent = trigger.entity;
    let Ok((mesh_doc, compiled_mesh, mesh_id)) = mesh_query.get(mesh_ent) else {
        return;
    };
    let Ok(maps) = entity_maps.get(mesh_doc.0) else {
        return;
    };

    for (node_ent, mesh_ref, node_doc) in &node_refs {
        if node_doc.0 != mesh_doc.0 || mesh_ref.0 != mesh_id.0 {
            continue;
        }
        if let Ok(mut entity_cmd) = commands.get_entity(node_ent) {
            entity_cmd.insert(Mesh3d(compiled_mesh.0.clone()));
        }
        assign_material(
            node_ent,
            maps,
            &compiled_mats,
            &mat_refs,
            &asset_server,
            &mut commands,
            &mut default_material,
        );
    }
}

pub(crate) fn on_material_compiled(
    trigger: On<Add, CompiledMaterial>,
    mat_query: Query<(&HsdChild, &CompiledMaterial, &MaterialId)>,
    node_refs: Query<(Entity, &MaterialRef, &HsdChild), With<NodeId>>,
    mut commands: Commands,
) {
    let mat_ent = trigger.entity;
    let Ok((mat_doc, compiled_mat, mat_id)) = mat_query.get(mat_ent) else {
        return;
    };

    for (node_ent, mat_ref, node_doc) in &node_refs {
        if node_doc.0 != mat_doc.0 || mat_ref.0 != mat_id.0 {
            continue;
        }
        if let Ok(mut entity_cmd) = commands.get_entity(node_ent) {
            entity_cmd.insert(MeshMaterial3d(compiled_mat.0.clone()));
        }
    }
}

fn assign_material(
    node_ent: Entity,
    maps: &HsdEntityMaps,
    compiled_mats: &Query<&CompiledMaterial>,
    mat_refs: &Query<&MaterialRef>,
    asset_server: &AssetServer,
    commands: &mut Commands,
    default_material: &mut Option<Handle<StandardMaterial>>,
) {
    let Ok(mut entity_cmd) = commands.get_entity(node_ent) else {
        return;
    };
    if let Ok(mat_ref) = mat_refs.get(node_ent) {
        let mat_ent = maps.materials.get(&mat_ref.0).copied();
        if let Some(mat_ent) = mat_ent
            && let Ok(compiled_mat) = compiled_mats.get(mat_ent)
        {
            entity_cmd.insert(MeshMaterial3d(compiled_mat.0.clone()));
        }
    } else {
        let mat = default_material
            .get_or_insert_with(|| asset_server.add(StandardMaterial::default()))
            .clone();
        entity_cmd.insert(MeshMaterial3d(mat));
    }
}

fn node_entity(
    docs: &Query<(Entity, &HsdRecordId)>,
    entity_maps: &Query<&HsdEntityMaps>,
    doc_id: &blake3::Hash,
    id: &TreeID,
) -> Option<Entity> {
    let (doc_ent, _) = docs.iter().find(|(_, r)| r.0 == *doc_id)?;
    let maps = entity_maps.get(doc_ent).ok()?;
    maps.nodes.get(id).copied()
}

fn is_transform_hierarchy_degenerate(
    start: Entity,
    q: &Query<(Option<&ChildOf>, Option<&Transform>)>,
    epsilon: f32,
) -> bool {
    let mut curr = start;
    while let Ok((child_of, maybe_t)) = q.get(curr) {
        if let Some(t) = maybe_t {
            let s = t.scale;
            if s.x.is_nan()
                || s.y.is_nan()
                || s.z.is_nan()
                || s.x.abs() < epsilon
                || s.y.abs() < epsilon
                || s.z.abs() < epsilon
            {
                return true;
            }
        }
        match child_of {
            Some(c) => curr = c.0,
            None => break,
        }
    }
    false
}

pub fn guard_physics_scale(
    query: Query<(
        Entity,
        &HsdNodePhysics,
        Has<Collider>,
        Has<RigidBody>,
        Has<Sensor>,
    )>,
    ancestors: Query<(Option<&ChildOf>, Option<&Transform>)>,
    mut commands: Commands,
) {
    for (ent, physics, has_collider, has_rigid_body, has_sensor) in &query {
        const EPSILON: f32 = 1e-5;

        let degenerate = is_transform_hierarchy_degenerate(ent, &ancestors, EPSILON);
        if degenerate {
            if has_collider || has_rigid_body {
                commands
                    .entity(ent)
                    .remove::<Collider>()
                    .remove::<ColliderParams>()
                    .remove::<RigidBody>()
                    .remove::<AngularInertia>()
                    .remove::<ComputedAngularInertia>()
                    .remove::<Sensor>();
            }
        } else {
            if !has_collider && let Some(ref c) = physics.collider {
                insert_collider(ent, c, &mut commands);
                // Bare colliders (no RigidBody) must be sensors so Avian's island
                // solver is never involved — preventing panics from static-static
                // or "not in island" contact pairs that scripts can inadvertently
                // create.
                if physics.rigid_body.is_none() && !has_sensor {
                    commands.entity(ent).insert(Sensor);
                }
            }
            if !has_rigid_body && let Some(ref rb) = physics.rigid_body {
                insert_rigid_body(ent, rb, &mut commands);
            }
        }
    }
}

pub(crate) fn node_transform(data: &HsdNode) -> Transform {
    let mut t = Transform::default();
    if let Some(tr) = &data.translation
        && tr.len() >= 3
    {
        t.translation = Vec3::new(tr[0] as f32, tr[1] as f32, tr[2] as f32);
    }
    if let Some(r) = &data.rotation
        && r.len() >= 4
    {
        t.rotation = Quat::from_xyzw(r[0] as f32, r[1] as f32, r[2] as f32, r[3] as f32);
    }
    if let Some(s) = &data.scale
        && s.len() >= 3
    {
        t.scale = Vec3::new(s[0] as f32, s[1] as f32, s[2] as f32);
    }
    t
}
