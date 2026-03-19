use avian3d::prelude::{AngularDamping, Collider, LinearDamping, RigidBody};
use bevy::prelude::*;
use smol_str::SmolStr;

use crate::{
    CompiledMaterial, CompiledMesh, HsdChild, HsdScripts, MaterialRef, MeshRef, NodeId,
    cache::SceneRegistry,
    data::{HsdCollider, HsdNodeData, HsdRigidBody},
};

use super::collider::insert_collider;

#[derive(Event)]
pub struct HsdNodeColliderSet {
    pub doc: Entity,
    pub id: SmolStr,
    pub collider: Option<HsdCollider>,
}

#[derive(Event)]
pub struct HsdNodeDespawned {
    pub doc: Entity,
    pub id: SmolStr,
}

#[derive(Event)]
pub struct HsdNodeMaterialSet {
    pub doc: Entity,
    pub id: SmolStr,
    pub material: Option<SmolStr>,
}

#[derive(Event)]
pub struct HsdNodeMeshSet {
    pub doc: Entity,
    pub id: SmolStr,
    pub mesh: Option<SmolStr>,
}

#[derive(Event)]
pub struct HsdNodeNameSet {
    pub doc: Entity,
    pub id: SmolStr,
    pub name: Option<String>,
}

#[derive(Event)]
pub struct HsdNodeParentSet {
    pub doc: Entity,
    pub id: SmolStr,
    pub parent: Option<SmolStr>,
}

#[derive(Event)]
pub struct HsdNodeRigidBodySet {
    pub doc: Entity,
    pub id: SmolStr,
    pub rigid_body: Option<HsdRigidBody>,
}

#[derive(Event)]
pub struct HsdNodeScriptsSet {
    pub doc: Entity,
    pub id: SmolStr,
    pub scripts: Vec<blake3::Hash>,
}

#[derive(Event)]
pub struct HsdNodeSpawned {
    pub doc: Entity,
    pub id: SmolStr,
}

#[derive(Event)]
pub struct HsdNodeTransformSet {
    pub doc: Entity,
    pub id: SmolStr,
    pub transform: Transform,
}

pub(crate) fn handle_hsd_node_spawned(
    trigger: On<HsdNodeSpawned>,
    registries: Query<&SceneRegistry>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, "node spawned");
    let Ok(registry) = registries.get(ev.doc) else {
        return;
    };
    let inner = registry
        .0
        .node_map
        .lock()
        .expect("node_map lock")
        .get(&ev.id)
        .cloned();
    let Some(inner) = inner else { return };
    if inner.entity.lock().expect("entity lock").is_some() {
        return;
    }
    let ent = commands
        .spawn((
            HsdChild { doc: ev.doc },
            NodeId(ev.id.clone()),
            Transform::IDENTITY,
        ))
        .id();
    *inner.entity.lock().expect("entity lock") = Some(ent);
}

pub(crate) fn handle_hsd_node_despawned(
    trigger: On<HsdNodeDespawned>,
    registries: Query<&SceneRegistry>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, "node despawned");
    let Ok(registry) = registries.get(ev.doc) else {
        return;
    };
    let mut node_map = registry.0.node_map.lock().expect("node_map lock");
    let Some(inner) = node_map.remove(&ev.id) else {
        return;
    };
    if inner.id != ev.id {
        node_map.remove(&inner.id);
    }
    drop(node_map);
    registry
        .0
        .nodes
        .lock()
        .expect("nodes lock")
        .retain(|n| n.id != inner.id);
    let ent = *inner.entity.lock().expect("entity lock");
    if let Some(ent) = ent {
        commands.entity(ent).despawn();
    }
}

pub(crate) fn handle_hsd_node_collider_set(
    trigger: On<HsdNodeColliderSet>,
    registries: Query<&SceneRegistry>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, has_collider = ev.collider.is_some(), "node collider set");
    let Ok(registry) = registries.get(ev.doc) else {
        return;
    };
    let ent = registry
        .0
        .node_map
        .lock()
        .expect("node_map lock")
        .get(&ev.id)
        .and_then(|n| *n.entity.lock().expect("entity lock"));
    let Some(ent) = ent else { return };
    commands
        .entity(ent)
        .remove::<Collider>()
        .remove::<super::collider::ColliderParams>();
    if let Some(ref c) = ev.collider {
        insert_collider(ent, c, &mut commands);
    }
}

pub(crate) fn handle_hsd_node_material_set(
    trigger: On<HsdNodeMaterialSet>,
    registries: Query<&SceneRegistry>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, material = ?ev.material, "node material set");
    let Ok(registry) = registries.get(ev.doc) else {
        return;
    };
    let ent = registry
        .0
        .node_map
        .lock()
        .expect("node_map lock")
        .get(&ev.id)
        .and_then(|n| *n.entity.lock().expect("entity lock"));
    let Some(ent) = ent else { return };
    let mut ecmd = commands.entity(ent);
    ecmd.remove::<MaterialRef>();
    if let Some(ref id) = ev.material {
        ecmd.insert(MaterialRef(id.clone()));
    }
}

pub(crate) fn handle_hsd_node_mesh_set(
    trigger: On<HsdNodeMeshSet>,
    registries: Query<&SceneRegistry>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, mesh = ?ev.mesh, "node mesh set");
    let Ok(registry) = registries.get(ev.doc) else {
        return;
    };
    let ent = registry
        .0
        .node_map
        .lock()
        .expect("node_map lock")
        .get(&ev.id)
        .and_then(|n| *n.entity.lock().expect("entity lock"));
    let Some(ent) = ent else { return };
    let mut ecmd = commands.entity(ent);
    ecmd.remove::<MeshRef>();
    if let Some(ref id) = ev.mesh {
        ecmd.insert(MeshRef(id.clone()));
    }
}

pub(crate) fn handle_hsd_node_name_set(
    trigger: On<HsdNodeNameSet>,
    registries: Query<&SceneRegistry>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, name = ?ev.name, "node name set");
    let Ok(registry) = registries.get(ev.doc) else {
        return;
    };
    let ent = registry
        .0
        .node_map
        .lock()
        .expect("node_map lock")
        .get(&ev.id)
        .and_then(|n| *n.entity.lock().expect("entity lock"));
    let Some(ent) = ent else { return };
    if let Some(ref name) = ev.name {
        commands.entity(ent).insert(Name::new(name.clone()));
    } else {
        commands.entity(ent).remove::<Name>();
    }
}

pub(crate) fn handle_hsd_node_parent_set(
    trigger: On<HsdNodeParentSet>,
    registries: Query<&SceneRegistry>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, parent = ?ev.parent, "node parent set");
    let Ok(registry) = registries.get(ev.doc) else {
        return;
    };
    let ent = registry
        .0
        .node_map
        .lock()
        .expect("node_map lock")
        .get(&ev.id)
        .and_then(|n| *n.entity.lock().expect("entity lock"));
    let Some(ent) = ent else { return };
    if let Some(ref pid) = ev.parent {
        let parent_ent = registry
            .0
            .node_map
            .lock()
            .expect("node_map lock")
            .get(pid)
            .and_then(|n| *n.entity.lock().expect("entity lock"));
        if let Some(parent_ent) = parent_ent {
            commands.entity(ent).insert(ChildOf(parent_ent));
        }
    } else {
        commands.entity(ent).remove::<ChildOf>();
    }
}

pub(crate) fn handle_hsd_node_rigid_body_set(
    trigger: On<HsdNodeRigidBodySet>,
    registries: Query<&SceneRegistry>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, kind = ?ev.rigid_body.as_ref().map(|r| &r.kind), "node rigid body set");
    let Ok(registry) = registries.get(ev.doc) else {
        return;
    };
    let ent = registry
        .0
        .node_map
        .lock()
        .expect("node_map lock")
        .get(&ev.id)
        .and_then(|n| *n.entity.lock().expect("entity lock"));
    let Some(ent) = ent else { return };
    commands.entity(ent).remove::<RigidBody>();
    let Some(ref data) = ev.rigid_body else {
        return;
    };

    let kind = match data.kind.as_str() {
        "dynamic" => RigidBody::Dynamic,
        "fixed" => RigidBody::Static,
        "kinematic" => RigidBody::Kinematic,
        other => {
            warn!("invalid rigid body kind: {other}");
            RigidBody::default()
        }
    };

    let mut ecmd = commands.entity(ent);
    ecmd.insert(kind);

    if kind == RigidBody::Dynamic {
        let linear = data.linear_damping.map_or(0.2, |v| v as f32);
        let angular = data.angular_damping.map_or(0.2, |v| v as f32);
        ecmd.insert((LinearDamping(linear), AngularDamping(angular)));
    }
}

pub(crate) fn handle_hsd_node_scripts_set(
    trigger: On<HsdNodeScriptsSet>,
    registries: Query<&SceneRegistry>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, count = ev.scripts.len(), "node scripts set");
    let Ok(registry) = registries.get(ev.doc) else {
        return;
    };
    let ent = registry
        .0
        .node_map
        .lock()
        .expect("node_map lock")
        .get(&ev.id)
        .and_then(|n| *n.entity.lock().expect("entity lock"));
    let Some(ent) = ent else { return };
    let mut ecmd = commands.entity(ent);
    if ev.scripts.is_empty() {
        ecmd.remove::<HsdScripts>();
    } else {
        ecmd.insert(HsdScripts(ev.scripts.clone()));
    }
}

pub(crate) fn handle_hsd_node_transform_set(
    trigger: On<HsdNodeTransformSet>,
    registries: Query<&SceneRegistry>,
    mut transforms: Query<&mut Transform>,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, "node transform set");
    let Ok(registry) = registries.get(ev.doc) else {
        return;
    };
    let ent = registry
        .0
        .node_map
        .lock()
        .expect("node_map lock")
        .get(&ev.id)
        .and_then(|n| *n.entity.lock().expect("entity lock"));
    let Some(ent) = ent else { return };
    if let Ok(mut t) = transforms.get_mut(ent) {
        *t = ev.transform;
    }
}

/// When a node gets a [`MeshRef`], try to link it to the compiled mesh.
pub(crate) fn on_mesh_ref_set(
    trigger: On<Add, MeshRef>,
    nodes: Query<(&MeshRef, &HsdChild), With<NodeId>>,
    registries: Query<&SceneRegistry>,
    compiled_meshes: Query<&CompiledMesh>,
    compiled_mats: Query<&CompiledMaterial>,
    mat_refs: Query<&MaterialRef>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut default_material: Local<Option<Handle<StandardMaterial>>>,
) {
    let node_ent = trigger.entity;
    debug!(entity = %node_ent, "mesh ref set");
    let Ok((mesh_ref, hsd_child)) = nodes.get(node_ent) else {
        return;
    };
    let Ok(registry) = registries.get(hsd_child.doc) else {
        return;
    };
    let mesh_ent = registry
        .0
        .meshes
        .lock()
        .expect("meshes lock")
        .get(&mesh_ref.0)
        .and_then(|inner| *inner.entity.lock().expect("entity lock"));
    let Some(mesh_ent) = mesh_ent else { return };
    let Ok(compiled_mesh) = compiled_meshes.get(mesh_ent) else {
        return;
    };
    commands
        .entity(node_ent)
        .insert(Mesh3d(compiled_mesh.0.clone()));
    assign_material(
        node_ent,
        registry,
        &compiled_mats,
        &mat_refs,
        &asset_server,
        &mut commands,
        &mut default_material,
    );
}

/// When a node loses its [`MeshRef`], remove [`Mesh3d`].
pub(crate) fn on_mesh_ref_removed(trigger: On<Remove, MeshRef>, mut commands: Commands) {
    debug!(entity = %trigger.entity, "mesh ref removed");
    commands.entity(trigger.entity).remove::<Mesh3d>();
}

/// When a mesh entity gets [`CompiledMesh`], assign [`Mesh3d`] to all referencing nodes.
pub(crate) fn on_mesh_compiled(
    trigger: On<Add, CompiledMesh>,
    mesh_query: Query<(&HsdChild, &CompiledMesh)>,
    node_refs: Query<(Entity, &MeshRef, &HsdChild), With<NodeId>>,
    registries: Query<&SceneRegistry>,
    compiled_mats: Query<&CompiledMaterial>,
    mat_refs: Query<&MaterialRef>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut default_material: Local<Option<Handle<StandardMaterial>>>,
) {
    let mesh_ent = trigger.entity;
    debug!(entity = %mesh_ent, "mesh compiled");
    let Ok((mesh_child, compiled_mesh)) = mesh_query.get(mesh_ent) else {
        return;
    };
    let Ok(registry) = registries.get(mesh_child.doc) else {
        return;
    };

    let mesh_id = {
        let meshes = registry.0.meshes.lock().expect("meshes lock");
        meshes
            .iter()
            .find(|(_, inner)| *inner.entity.lock().expect("entity lock") == Some(mesh_ent))
            .map(|(id, _)| id.clone())
    };
    let Some(mesh_id) = mesh_id else { return };

    for (node_ent, mesh_ref, node_child) in &node_refs {
        if node_child.doc != mesh_child.doc || mesh_ref.0 != mesh_id {
            continue;
        }
        commands
            .entity(node_ent)
            .insert(Mesh3d(compiled_mesh.0.clone()));
        assign_material(
            node_ent,
            registry,
            &compiled_mats,
            &mat_refs,
            &asset_server,
            &mut commands,
            &mut default_material,
        );
    }
}

/// When a material entity gets [`CompiledMaterial`], assign it to all referencing
/// nodes that already have a [`Mesh3d`].
pub(crate) fn on_material_compiled(
    trigger: On<Add, CompiledMaterial>,
    mat_query: Query<(&HsdChild, &CompiledMaterial)>,
    node_refs: Query<(Entity, &MaterialRef, &HsdChild), (With<NodeId>, With<Mesh3d>)>,
    registries: Query<&SceneRegistry>,
    mut commands: Commands,
) {
    let mat_ent = trigger.entity;
    debug!(entity = %mat_ent, "material compiled");
    let Ok((mat_child, compiled_mat)) = mat_query.get(mat_ent) else {
        return;
    };
    let Ok(registry) = registries.get(mat_child.doc) else {
        return;
    };

    let mat_id = {
        let materials = registry.0.materials.lock().expect("materials lock");
        materials
            .iter()
            .find(|(_, inner)| *inner.entity.lock().expect("entity lock") == Some(mat_ent))
            .map(|(id, _)| id.clone())
    };
    let Some(mat_id) = mat_id else { return };

    for (node_ent, mat_ref, node_child) in &node_refs {
        if node_child.doc != mat_child.doc || mat_ref.0 != mat_id {
            continue;
        }
        commands
            .entity(node_ent)
            .insert(MeshMaterial3d(compiled_mat.0.clone()));
    }
}

fn assign_material(
    node_ent: Entity,
    registry: &SceneRegistry,
    compiled_mats: &Query<&CompiledMaterial>,
    mat_refs: &Query<&MaterialRef>,
    asset_server: &AssetServer,
    commands: &mut Commands,
    default_material: &mut Option<Handle<StandardMaterial>>,
) {
    if let Ok(mat_ref) = mat_refs.get(node_ent) {
        let mat_ent = registry
            .0
            .materials
            .lock()
            .expect("materials lock")
            .get(&mat_ref.0)
            .and_then(|inner| *inner.entity.lock().expect("entity lock"));
        if let Some(mat_ent) = mat_ent
            && let Ok(cm) = compiled_mats.get(mat_ent)
        {
            commands
                .entity(node_ent)
                .insert(MeshMaterial3d(cm.0.clone()));
        }
    } else {
        let mat = default_material
            .get_or_insert_with(|| asset_server.add(StandardMaterial::default()))
            .clone();
        commands.entity(node_ent).insert(MeshMaterial3d(mat));
    }
}

pub(crate) fn node_transform(data: &HsdNodeData) -> Transform {
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
