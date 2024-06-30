use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
        render_asset::RenderAssetUsages,
    },
    utils::HashMap,
};
use bevy_xpbd_3d::prelude::*;

use super::{ActionReceiver, ScriptAction};

#[derive(Component, Clone, Copy, Debug)]
pub struct NodeId(pub u32);

#[derive(Bundle)]
pub struct WiredNodeBundle {
    pub id: NodeId,
    pub node_mesh: NodeMesh,
    pub spatial: SpatialBundle,
}

impl WiredNodeBundle {
    pub fn new(id: u32) -> Self {
        Self {
            id: NodeId(id),
            node_mesh: NodeMesh::default(),
            spatial: SpatialBundle::default(),
        }
    }
}

#[derive(Component, Default)]
pub struct NodeMesh {
    pub id: Option<u32>,
    pub primitives: HashMap<u32, Entity>,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct MeshId(pub u32);

#[derive(Bundle)]
pub struct WiredMeshBundle {
    pub id: MeshId,
}

impl WiredMeshBundle {
    pub fn new(id: u32) -> Self {
        Self { id: MeshId(id) }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct PrimitiveId(pub u32);

#[derive(Bundle)]
pub struct WiredPrimitiveBundle {
    pub handle_mesh: Handle<Mesh>,
    pub id: PrimitiveId,
    pub mesh: MeshId,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct MaterialId(pub u32);

#[derive(Bundle)]
pub struct WiredMaterialBundle {
    pub id: MaterialId,
    pub handle: Handle<StandardMaterial>,
}

impl WiredMaterialBundle {
    pub fn new(id: u32, handle: Handle<StandardMaterial>) -> Self {
        Self {
            id: MaterialId(id),
            handle,
        }
    }
}

pub fn handle_actions(
    world: &mut World,
    materials: &mut QueryState<(Entity, &MaterialId, &Handle<StandardMaterial>)>,
    meshes: &mut QueryState<(Entity, &MeshId), Without<PrimitiveId>>,
    mut default_material: Local<Option<Handle<StandardMaterial>>>,
    nodes: &mut QueryState<(Entity, &NodeId, &mut NodeMesh)>,
    primitives: &mut QueryState<(
        Entity,
        &PrimitiveId,
        &MeshId,
        &Handle<Mesh>,
        Option<&MaterialId>,
    )>,
    scripts: &mut QueryState<(Entity, &ActionReceiver)>,
    transforms: &mut QueryState<&mut Transform>,
) {
    if default_material.is_none() {
        let mut material_assets = world.resource_mut::<Assets<StandardMaterial>>();
        *default_material = Some(material_assets.add(StandardMaterial::default()));
    }

    let default_material = default_material.as_ref().unwrap();

    let scripts = scripts
        .iter(world)
        .map(|(e, r)| (e, r.0.clone()))
        .collect::<Vec<_>>();

    for (entity, receiver) in scripts {
        while let Ok(msg) = receiver.try_recv() {
            trace!("handling: {:?}", msg);

            match msg {
                ScriptAction::CreateMaterial { id } => {
                    let span = info_span!("CreateMaterial", id);
                    let s = span.entered();

                    if find_material(materials, id, world).is_some() {
                        warn!("Material {} already exists.", id);
                    } else {
                        let mut material_assets = world.resource_mut::<Assets<StandardMaterial>>();
                        let handle = material_assets.add(StandardMaterial::default());
                        world.spawn(WiredMaterialBundle::new(id, handle));
                    }

                    drop(s);
                }
                ScriptAction::CreateMesh { id } => {
                    let span = info_span!("CreateMesh", id);
                    let s = span.entered();

                    if find_mesh(meshes, id, world).is_some() {
                        warn!("Mesh {} already exists.", id);
                    } else {
                        world.spawn(WiredMeshBundle::new(id));
                    }

                    drop(s);
                }
                ScriptAction::CreateNode { id } => {
                    let span = info_span!("CreateNode", id);
                    let s = span.entered();

                    if find_node(nodes, id, world).is_some() {
                        warn!("Node {} already exists.", id);
                    } else {
                        let node_ent = world.spawn(WiredNodeBundle::new(id)).id();
                        world.entity_mut(entity).add_child(node_ent);
                    }

                    drop(s);
                }
                ScriptAction::CreatePrimitive { id, mesh } => {
                    let span = info_span!("CreatePrimitive", id, mesh);
                    let s = span.entered();

                    if find_mesh(meshes, mesh, world).is_none() {
                        warn!("Mesh {} not found.", mesh);
                    } else if find_primitive(primitives, id, world).is_some() {
                        warn!("Primitive {} already exists.", id);
                    } else {
                        let mut mesh_assets = world.resource_mut::<Assets<Mesh>>();
                        let mesh_handle = mesh_assets.add(Mesh::new(
                            PrimitiveTopology::TriangleList,
                            RenderAssetUsages::all(),
                        ));

                        world.spawn(WiredPrimitiveBundle {
                            id: PrimitiveId(id),
                            mesh: MeshId(mesh),
                            handle_mesh: mesh_handle.clone(),
                        });

                        // Update any nodes using this mesh.
                        let mut to_adds = Vec::new();

                        for (node_ent, _, node_mesh) in nodes.iter(world) {
                            if node_mesh.id == Some(mesh) {
                                let material_id = None;
                                let to_add = vec![(id, mesh_handle.clone(), material_id)];
                                to_adds.push((node_ent, to_add));
                            }
                        }

                        for (node_ent, to_add) in to_adds {
                            let new_primitives = spawn_primitives(
                                node_ent,
                                to_add,
                                materials,
                                default_material,
                                world,
                            );

                            let (_, _, mut node_mesh) = nodes.get_mut(world, node_ent).unwrap();

                            for (id, value) in new_primitives {
                                node_mesh.primitives.insert(id, value);
                            }
                        }
                    }

                    drop(s);
                }
                ScriptAction::RemoveMaterial { id } => {
                    let span = info_span!("RemoveMaterial", id);
                    let s = span.entered();

                    if let Some((ent, ..)) = find_material(materials, id, world) {
                        world.entity_mut(ent).despawn_recursive();
                    } else {
                        warn!("Material {} does not exist", id);
                    }

                    drop(s);
                }
                ScriptAction::RemoveMesh { id } => {
                    let span = info_span!("RemoveMesh", id);
                    let s = span.entered();

                    if let Some(ent) = find_mesh(meshes, id, world) {
                        world.entity_mut(ent).despawn_recursive();
                    } else {
                        warn!("Mesh {} does not exist", id);
                    }

                    drop(s);
                }
                ScriptAction::RemoveNode { id } => {
                    let span = info_span!("RemoveNode", id);
                    let s = span.entered();

                    if let Some((ent, ..)) = find_node(nodes, id, world) {
                        world.entity_mut(ent).despawn_recursive();
                    } else {
                        warn!("Node {} does not exist", id);
                    }

                    drop(s);
                }
                ScriptAction::RemovePrimitive { id, mesh } => {
                    let span = info_span!("RemovePrimitive", id);
                    let s = span.entered();

                    if let Some((ent, ..)) = find_primitive(primitives, id, world) {
                        world.entity_mut(ent).despawn_recursive();

                        // Update any nodes using this mesh.
                        let mut to_remove = Vec::new();

                        for (_, _, mut node_mesh) in nodes.iter_mut(world) {
                            if node_mesh.id == Some(mesh) {
                                let ent = node_mesh.primitives.remove(&id).unwrap();
                                to_remove.push(ent);
                            }
                        }

                        for ent in to_remove {
                            world.entity_mut(ent).despawn_recursive();
                        }
                    } else {
                        warn!("Primitive {} does not exist", id);
                    }

                    drop(s);
                }
                ScriptAction::SetMaterialColor { id, color } => {
                    let span = info_span!("SetMaterialColor", id);
                    let s = span.entered();

                    if let Some((_, handle)) = find_material(materials, id, world) {
                        let handle = handle.clone();
                        let mut material_assets = world.resource_mut::<Assets<StandardMaterial>>();
                        let material = material_assets.get_mut(handle).unwrap();

                        material.base_color = color;
                    }

                    drop(s);
                }
                ScriptAction::SetNodeCollider { id, collider } => {
                    let span = info_span!("SetNodeCollider", id);
                    let s = span.entered();

                    if let Some((ent, ..)) = find_node(nodes, id, world) {
                        if let Some(collider) = collider {
                            world.entity_mut(ent).insert(collider);
                        } else {
                            world.entity_mut(ent).remove::<Collider>();
                        }
                    } else {
                        warn!("Node {} does not exist", id);
                    }

                    drop(s);
                }
                ScriptAction::SetNodeParent { id, parent } => {
                    let span = info_span!("SetNodeParent", id, parent);
                    let s = span.entered();

                    if let Some((ent, ..)) = find_node(nodes, id, world) {
                        if let Some(parent) = parent {
                            if let Some((parent_ent, ..)) = find_node(nodes, parent, world) {
                                world.entity_mut(parent_ent).push_children(&[ent]);
                            } else {
                                world.entity_mut(ent).remove_parent();
                            }
                        } else {
                            world.entity_mut(ent).remove_parent();
                        }
                    } else {
                        warn!("Node {} does not exist", id);
                    }

                    drop(s);
                }
                ScriptAction::SetNodeRigidBody { id, rigid_body } => {
                    let span = info_span!("SetNodeRigidBody", id);
                    let s = span.entered();

                    if let Some((ent, ..)) = find_node(nodes, id, world) {
                        if let Some(rigid_body) = rigid_body {
                            world.entity_mut(ent).insert(rigid_body);
                        } else {
                            world.entity_mut(ent).remove::<RigidBody>();
                        }
                    } else {
                        warn!("Node {} does not exist", id);
                    }

                    drop(s);
                }
                ScriptAction::SetNodeTransform { id, transform } => {
                    let span = info_span!("SetNodeTransform", id);
                    let s = span.entered();

                    if let Some((ent, ..)) = find_node(nodes, id, world) {
                        let mut node_transform = transforms.get_mut(world, ent).unwrap();
                        node_transform.clone_from(&transform);
                    }

                    drop(s);
                }
                ScriptAction::SetNodeMesh { id, mesh } => {
                    let span = info_span!("SetNodeMesh", id, mesh);
                    let s = span.entered();

                    let primitives = mesh.map(|mesh| {
                        primitives
                            .iter(world)
                            .filter_map(|(_, pid, m, handle_mesh, material)| {
                                if m.0 == mesh {
                                    Some((pid.0, handle_mesh.clone(), material.copied()))
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                    });

                    if let Some((node_ent, mut node_mesh)) = find_node(nodes, id, world) {
                        let mut to_add = Vec::new();
                        let mut to_remove = Vec::new();

                        node_mesh.id = mesh;

                        if mesh.is_some() {
                            let primitives = primitives.unwrap();

                            let mut to_remove_ids = Vec::new();

                            for id in node_mesh.primitives.keys() {
                                if !primitives.iter().any(|(pid, ..)| pid == id) {
                                    to_remove_ids.push(*id);
                                }
                            }

                            for id in to_remove_ids {
                                let ent = node_mesh.primitives.remove(&id).unwrap();
                                to_remove.push(ent);
                            }

                            for (id, handle_mesh, material) in primitives {
                                if node_mesh.primitives.contains_key(&id) {
                                    continue;
                                }

                                to_add.push((id, handle_mesh.clone(), material));
                            }
                        } else {
                            for ent in node_mesh.primitives.values() {
                                to_remove.push(*ent);
                            }

                            node_mesh.primitives.clear();
                        }

                        drop(node_mesh);

                        for ent in to_remove {
                            world.entity_mut(ent).despawn();
                        }

                        let new_primitives =
                            spawn_primitives(node_ent, to_add, materials, default_material, world);

                        let (_, mut node_mesh) = find_node(nodes, id, world).unwrap();

                        for (id, primitive_ent) in new_primitives {
                            node_mesh.primitives.insert(id, primitive_ent);
                        }
                    } else {
                        warn!("Node {} does not exist", id);
                    }

                    drop(s);
                }
                ScriptAction::SetPrimitiveIndices { id, value } => {
                    let span = info_span!("SetPrimitiveIndices", id);
                    let s = span.entered();

                    if let Some((_, handle_mesh, _)) = find_primitive(primitives, id, world) {
                        let handle = handle_mesh.clone();
                        let mut mesh_assets = world.resource_mut::<Assets<Mesh>>();
                        let mesh = mesh_assets.get_mut(handle).unwrap();
                        mesh.insert_indices(Indices::U32(value));
                    } else {
                        warn!("Primitive {} does not exist", id);
                    }

                    drop(s);
                }
                ScriptAction::SetPrimitiveMaterial { id, material } => {
                    let span = info_span!("SetPrimitiveMaterial", id, material);
                    let s = span.entered();

                    if let Some((ent, ..)) = find_primitive(primitives, id, world) {
                        let material_handle = if let Some(material) = material {
                            if let Some((_, handle)) = find_material(materials, material, world) {
                                handle.clone()
                            } else {
                                warn!("Material {} does not exist", material);
                                default_material.clone()
                            }
                        } else {
                            default_material.clone()
                        };

                        world.entity_mut(ent).insert(material_handle.clone());

                        // Update all nodes using this primitive.
                        let mut to_update = Vec::new();

                        for (_, _, node_mesh) in nodes.iter(world) {
                            if let Some(ent) = node_mesh.primitives.get(&id) {
                                to_update.push(*ent);
                            }
                        }

                        for ent in to_update {
                            world.entity_mut(ent).insert(material_handle.clone());
                        }
                    } else {
                        warn!("Primitive {} does not exist", id);
                    }

                    drop(s);
                }
                ScriptAction::SetPrimitiveNormals { id, value } => {
                    let span = info_span!("SetPrimitiveNormals", id);
                    let s = span.entered();

                    if let Some((_, handle_mesh, _)) = find_primitive(primitives, id, world) {
                        let handle = handle_mesh.clone();
                        let mut mesh_assets = world.resource_mut::<Assets<Mesh>>();
                        let mesh = mesh_assets.get_mut(handle).unwrap();

                        let value = value
                            .chunks(3)
                            .map(|x| [x[0], x[1], x[2]])
                            .collect::<Vec<_>>();

                        mesh.insert_attribute(
                            Mesh::ATTRIBUTE_NORMAL,
                            VertexAttributeValues::Float32x3(value),
                        );
                    } else {
                        warn!("Primitive {} does not exist", id);
                    }

                    drop(s);
                }
                ScriptAction::SetPrimitivePositions { id, value } => {
                    let span = info_span!("SetPrimitivePositions", id);
                    let s = span.entered();

                    if let Some((_, handle_mesh, _)) = find_primitive(primitives, id, world) {
                        let handle = handle_mesh.clone();
                        let mut mesh_assets = world.resource_mut::<Assets<Mesh>>();
                        let mesh = mesh_assets.get_mut(handle).unwrap();

                        let value = value
                            .chunks(3)
                            .map(|x| [x[0], x[1], x[2]])
                            .collect::<Vec<_>>();

                        mesh.insert_attribute(
                            Mesh::ATTRIBUTE_POSITION,
                            VertexAttributeValues::Float32x3(value),
                        );
                    } else {
                        warn!("Primitive {} does not exist", id);
                    }

                    drop(s);
                }
                ScriptAction::SetPrimitiveUvs { id, value } => {
                    let span = info_span!("SetPrimitiveUvs", id);
                    let s = span.entered();

                    if let Some((_, handle_mesh, _)) = find_primitive(primitives, id, world) {
                        let handle = handle_mesh.clone();
                        let mut mesh_assets = world.resource_mut::<Assets<Mesh>>();
                        let mesh = mesh_assets.get_mut(handle).unwrap();

                        if value.len() % 2 != 0 {
                            warn!("UVs do not have an even length! Got: {}", value.len());
                        } else {
                            let value = value.chunks(2).map(|x| [x[0], x[1]]).collect::<Vec<_>>();

                            mesh.insert_attribute(
                                Mesh::ATTRIBUTE_UV_0,
                                VertexAttributeValues::Float32x2(value),
                            );
                        }
                    } else {
                        warn!("Primitive {} does not exist", id);
                    }

                    drop(s);
                }
            };
        }
    }
}

fn spawn_primitives(
    node_ent: Entity,
    to_add: Vec<(u32, Handle<Mesh>, Option<MaterialId>)>,
    materials: &mut QueryState<(Entity, &MaterialId, &Handle<StandardMaterial>)>,
    default_material: &Handle<StandardMaterial>,
    world: &mut World,
) -> Vec<(u32, Entity)> {
    let mut new_primitives = Vec::new();

    for (id, mesh_handle, material_id) in to_add {
        let material = if let Some(material_id) = material_id {
            if let Some((_, handle_material)) = find_material(materials, material_id.0, world) {
                warn!("Material {} set", material_id.0);
                handle_material.clone()
            } else {
                warn!("Material {} does not exist", material_id.0);
                default_material.clone()
            }
        } else {
            default_material.clone()
        };

        let primitive_ent = world
            .spawn(MaterialMeshBundle {
                material,
                mesh: mesh_handle.clone(),
                ..Default::default()
            })
            .id();

        world.entity_mut(node_ent).add_child(primitive_ent);

        new_primitives.push((id, primitive_ent));
    }

    new_primitives
}

fn find_node<'a>(
    nodes: &'a mut QueryState<(Entity, &NodeId, &mut NodeMesh)>,
    id: u32,
    world: &'a mut World,
) -> Option<(Entity, Mut<'a, NodeMesh>)> {
    nodes.iter_mut(world).find_map(|(ent, nid, primitives)| {
        if nid.0 == id {
            Some((ent, primitives))
        } else {
            None
        }
    })
}

fn find_mesh(
    meshes: &mut QueryState<(Entity, &MeshId), Without<PrimitiveId>>,
    id: u32,
    world: &World,
) -> Option<Entity> {
    meshes
        .iter(world)
        .find_map(|(ent, mid)| if mid.0 == id { Some(ent) } else { None })
}

fn find_primitive<'a>(
    primitives: &'a mut QueryState<(
        Entity,
        &PrimitiveId,
        &MeshId,
        &Handle<Mesh>,
        Option<&MaterialId>,
    )>,
    id: u32,
    world: &'a World,
) -> Option<(Entity, &'a Handle<Mesh>, Option<&'a MaterialId>)> {
    primitives
        .iter(world)
        .find_map(|(ent, pid, _, handle_mesh, material)| {
            if pid.0 == id {
                Some((ent, handle_mesh, material))
            } else {
                None
            }
        })
}

fn find_material<'a>(
    materials: &'a mut QueryState<(Entity, &MaterialId, &Handle<StandardMaterial>)>,
    id: u32,
    world: &'a World,
) -> Option<(Entity, &'a Handle<StandardMaterial>)> {
    materials.iter(world).find_map(|(ent, mid, handle)| {
        if mid.0 == id {
            Some((ent, handle))
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use crossbeam::channel::Sender;
    use tracing_test::traced_test;

    use super::*;

    fn setup_test() -> (App, Sender<ScriptAction>) {
        let mut app = App::new();

        app.add_plugins(AssetPlugin::default())
            .init_asset::<Mesh>()
            .init_asset::<StandardMaterial>()
            .add_systems(Update, handle_actions);

        let (send, recv) = crossbeam::channel::unbounded();
        app.world.spawn(ActionReceiver(recv));

        (app, send)
    }

    // Node
    #[test]
    #[traced_test]
    fn create_node() {
        let (mut app, send) = setup_test();

        let id = 0;
        send.send(ScriptAction::CreateNode { id }).unwrap();
        app.update();

        let mut nodes = app.world.query::<&NodeId>();
        let found_id = nodes.single(&app.world);
        assert_eq!(found_id.0, id);
    }

    #[test]
    #[traced_test]
    fn create_node_duplicate_id() {
        let (mut app, send) = setup_test();

        let id = 0;
        app.world.spawn(WiredNodeBundle::new(id));

        send.send(ScriptAction::CreateNode { id }).unwrap();
        app.update();

        let mut nodes = app.world.query::<&NodeId>();
        assert!(nodes.iter(&app.world).len() == 1);
    }

    #[test]
    #[traced_test]
    fn remove_node() {
        let (mut app, send) = setup_test();

        let id = 0;
        let ent = app.world.spawn(WiredNodeBundle::new(id)).id();

        send.send(ScriptAction::RemoveNode { id }).unwrap();
        app.update();

        assert!(app.world.get_entity(ent).is_none());
    }

    #[test]
    #[traced_test]
    fn remove_invalid_node() {
        let (mut app, send) = setup_test();

        send.send(ScriptAction::RemoveNode { id: 0 }).unwrap();
        app.update();
    }

    #[test]
    #[traced_test]
    fn set_node_parent_some() {
        let (mut app, send) = setup_test();

        let parent_id = 0;
        let parent_ent = app.world.spawn(WiredNodeBundle::new(parent_id)).id();

        let child_id = 1;
        let child_ent = app.world.spawn(WiredNodeBundle::new(child_id)).id();

        send.send(ScriptAction::SetNodeParent {
            id: child_id,
            parent: Some(parent_id),
        })
        .unwrap();
        app.update();

        let children = app.world.get::<Children>(parent_ent).unwrap();
        assert!(children.contains(&child_ent));
    }

    #[test]
    #[traced_test]
    fn set_node_parent_none() {
        let (mut app, send) = setup_test();

        let parent_id = 0;
        let parent_ent = app.world.spawn(WiredNodeBundle::new(parent_id)).id();

        let child_id = 1;
        let mut child_ent = None;
        app.world.entity_mut(parent_ent).with_children(|builder| {
            child_ent = Some(builder.spawn(WiredNodeBundle::new(child_id)).id());
        });
        let child_ent = child_ent.unwrap();

        let children = app.world.get::<Children>(parent_ent).unwrap();
        assert!(children.contains(&child_ent));

        send.send(ScriptAction::SetNodeParent {
            id: child_id,
            parent: None,
        })
        .unwrap();
        app.update();

        assert!(app.world.get::<Children>(parent_ent).is_none());
    }

    #[test]
    #[traced_test]
    fn set_node_invalid_parent() {
        let (mut app, send) = setup_test();

        let parent_id = 0;
        let child_id = 1;
        app.world.spawn(WiredNodeBundle::new(child_id));

        send.send(ScriptAction::SetNodeParent {
            id: child_id,
            parent: Some(parent_id),
        })
        .unwrap();
        app.update();
    }

    #[test]
    #[traced_test]
    fn set_node_transform() {
        let (mut app, send) = setup_test();

        let id = 0;
        let ent = app.world.spawn(WiredNodeBundle::new(id)).id();

        let transform = Transform {
            translation: Vec3::splat(1.0),
            rotation: Quat::from_xyzw(0.1, 0.2, 0.3, 0.4),
            scale: Vec3::splat(3.0),
        };

        send.send(ScriptAction::SetNodeTransform { id, transform })
            .unwrap();
        app.update();

        assert_eq!(app.world.get::<Transform>(ent).unwrap(), &transform);
    }

    #[test]
    #[traced_test]
    fn set_node_mesh_some() {
        let (mut app, send) = setup_test();

        let id = 0;
        let ent = app.world.spawn(WiredNodeBundle::new(id)).id();

        let mesh_id = 1;
        app.world.spawn(WiredMeshBundle::new(mesh_id));

        let material_id = 2;
        let handle_material = Handle::default();
        app.world.spawn(WiredMaterialBundle {
            id: MaterialId(material_id),
            handle: handle_material.clone(),
        });

        let handle_mesh = Handle::default();
        let primitive_id = 3;
        app.world.spawn((
            WiredPrimitiveBundle {
                handle_mesh: handle_mesh.clone(),
                id: PrimitiveId(primitive_id),
                mesh: MeshId(mesh_id),
            },
            MaterialId(material_id),
        ));

        send.send(ScriptAction::SetNodeMesh {
            id,
            mesh: Some(mesh_id),
        })
        .unwrap();
        app.update();

        let node_mesh = app.world.get::<NodeMesh>(ent).unwrap();
        assert_eq!(node_mesh.id, Some(mesh_id));

        let primitive_ent = node_mesh.primitives[&primitive_id];

        let found_mesh = app.world.get::<Handle<Mesh>>(primitive_ent).unwrap();
        assert_eq!(*found_mesh, handle_mesh);

        let found_material = app
            .world
            .get::<Handle<StandardMaterial>>(primitive_ent)
            .unwrap();
        assert_eq!(*found_material, handle_material);
    }

    #[test]
    #[traced_test]
    fn set_node_mesh_none() {
        let (mut app, send) = setup_test();

        let mesh_id = 1;
        app.world.spawn(WiredMeshBundle::new(mesh_id));

        let handle = Handle::default();
        let primitive_id = 2;
        app.world.spawn(WiredPrimitiveBundle {
            handle_mesh: handle.clone(),
            id: PrimitiveId(primitive_id),
            mesh: MeshId(mesh_id),
        });

        let primitive_ent = app
            .world
            .spawn(MaterialMeshBundle::<StandardMaterial> {
                mesh: handle,
                ..Default::default()
            })
            .id();

        let mut node_mesh = NodeMesh::default();
        node_mesh.primitives.insert(primitive_id, primitive_ent);

        let id = 0;
        let ent = app
            .world
            .spawn(WiredNodeBundle {
                id: NodeId(id),
                spatial: SpatialBundle::default(),
                node_mesh,
            })
            .id();

        send.send(ScriptAction::SetNodeMesh { id, mesh: None })
            .unwrap();
        app.update();

        let node_mesh = app.world.get::<NodeMesh>(ent).unwrap();
        assert!(node_mesh.id.is_none());
        assert!(node_mesh.primitives.is_empty());

        assert!(app.world.get_entity(primitive_ent).is_none());
    }

    #[test]
    #[traced_test]
    fn set_node_collider_some() {
        let (mut app, send) = setup_test();

        let id = 0;
        let ent = app.world.spawn(WiredNodeBundle::new(id)).id();

        let collider = Collider::sphere(0.5);

        send.send(ScriptAction::SetNodeCollider {
            id,
            collider: Some(collider.clone()),
        })
        .unwrap();
        app.update();

        assert!(app.world.get::<Collider>(ent).is_some());
    }

    #[test]
    #[traced_test]
    fn set_node_collider_none() {
        let (mut app, send) = setup_test();

        let id = 0;
        let ent = app
            .world
            .spawn((WiredNodeBundle::new(id), Collider::sphere(0.5)))
            .id();

        send.send(ScriptAction::SetNodeCollider { id, collider: None })
            .unwrap();
        app.update();

        assert!(app.world.get::<Collider>(ent).is_none());
    }

    #[test]
    #[traced_test]
    fn set_node_rigid_body_some() {
        let (mut app, send) = setup_test();

        let id = 0;
        let ent = app.world.spawn(WiredNodeBundle::new(id)).id();

        let rigid_body = RigidBody::Dynamic;

        send.send(ScriptAction::SetNodeRigidBody {
            id,
            rigid_body: Some(rigid_body.clone()),
        })
        .unwrap();
        app.update();

        assert!(app.world.get::<RigidBody>(ent).is_some());
    }

    #[test]
    #[traced_test]
    fn set_node_rigid_body_none() {
        let (mut app, send) = setup_test();

        let id = 0;
        let ent = app
            .world
            .spawn((WiredNodeBundle::new(id), RigidBody::Dynamic))
            .id();

        send.send(ScriptAction::SetNodeRigidBody {
            id,
            rigid_body: None,
        })
        .unwrap();
        app.update();

        assert!(app.world.get::<RigidBody>(ent).is_none());
    }

    // Mesh
    #[test]
    #[traced_test]
    fn create_mesh() {
        let (mut app, send) = setup_test();

        let id = 0;
        send.send(ScriptAction::CreateMesh { id }).unwrap();
        app.update();

        let mut meshes = app.world.query::<&MeshId>();
        let found_id = meshes.single(&app.world);
        assert_eq!(found_id.0, id);
    }

    #[test]
    #[traced_test]
    fn create_mesh_duplicate_id() {
        let (mut app, send) = setup_test();

        let id = 0;
        app.world.spawn(MeshId(id));

        send.send(ScriptAction::CreateMesh { id }).unwrap();
        app.update();

        let mut meshes = app.world.query::<&MeshId>();
        assert!(meshes.iter(&app.world).len() == 1);
    }

    #[test]
    #[traced_test]
    fn remove_mesh() {
        let (mut app, send) = setup_test();

        let id = 0;
        let ent = app.world.spawn(MeshId(id)).id();

        send.send(ScriptAction::RemoveMesh { id }).unwrap();
        app.update();

        assert!(app.world.get_entity(ent).is_none());
    }

    #[test]
    #[traced_test]
    fn remove_invalid_mesh() {
        let (mut app, send) = setup_test();

        send.send(ScriptAction::RemoveMesh { id: 0 }).unwrap();
        app.update();
    }

    // Primitive
    #[test]
    #[traced_test]
    fn create_primitive() {
        let (mut app, send) = setup_test();

        let mesh = 0;
        app.world.spawn(MeshId(mesh));

        let id = 1;
        send.send(ScriptAction::CreatePrimitive { id, mesh })
            .unwrap();
        app.update();

        let mut primitives = app.world.query::<&PrimitiveId>();
        let found_id = primitives.single(&app.world);
        assert_eq!(found_id.0, id);
    }

    #[test]
    #[traced_test]
    fn create_primitive_invalid_mesh() {
        let (mut app, send) = setup_test();

        let mesh = 0;
        let id = 1;
        send.send(ScriptAction::CreatePrimitive { id, mesh })
            .unwrap();
        app.update();

        let mut primitives = app.world.query::<&PrimitiveId>();
        assert_eq!(primitives.iter(&app.world).len(), 0);
    }

    #[test]
    #[traced_test]
    fn create_primitive_duplicate_id() {
        let (mut app, send) = setup_test();

        let mesh = 0;
        app.world.spawn(MeshId(mesh));

        let id = 1;
        app.world.spawn(WiredPrimitiveBundle {
            id: PrimitiveId(id),
            mesh: MeshId(mesh),
            handle_mesh: Default::default(),
        });

        send.send(ScriptAction::CreatePrimitive { id, mesh })
            .unwrap();
        app.update();

        let mut primitives = app.world.query::<&PrimitiveId>();
        assert!(primitives.iter(&app.world).len() == 1);
    }

    #[test]
    #[traced_test]
    fn remove_primitive() {
        let (mut app, send) = setup_test();

        let mesh = 0;
        app.world.spawn(MeshId(mesh));

        let id = 1;
        let ent = app
            .world
            .spawn(WiredPrimitiveBundle {
                id: PrimitiveId(id),
                mesh: MeshId(mesh),
                handle_mesh: Default::default(),
            })
            .id();

        send.send(ScriptAction::RemovePrimitive { id, mesh })
            .unwrap();
        app.update();

        assert!(app.world.get_entity(ent).is_none());
    }

    #[test]
    #[traced_test]
    fn remove_invalid_primitive() {
        let (mut app, send) = setup_test();

        send.send(ScriptAction::RemovePrimitive { id: 1, mesh: 0 })
            .unwrap();
        app.update();
    }

    #[test]
    #[traced_test]
    fn set_primitive_indices() {
        let (mut app, send) = setup_test();

        let mesh = 0;
        app.world.spawn(MeshId(mesh));

        let mut meshes = app.world.resource_mut::<Assets<Mesh>>();
        let handle = meshes.add(Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::all(),
        ));

        let id = 1;
        app.world.spawn(WiredPrimitiveBundle {
            id: PrimitiveId(id),
            mesh: MeshId(mesh),
            handle_mesh: handle.clone(),
        });

        let value = vec![0, 1, 2, 3, 4, 5];

        send.send(ScriptAction::SetPrimitiveIndices {
            id,
            value: value.clone(),
        })
        .unwrap();
        app.update();

        let meshes = app.world.resource::<Assets<Mesh>>();
        let mesh = meshes.get(handle).unwrap();

        let indices = match mesh.indices().unwrap() {
            Indices::U32(v) => v,
            _ => panic!(),
        };

        assert_eq!(indices.as_slice(), value.as_slice());
    }

    #[test]
    #[traced_test]
    fn set_primitive_normals() {
        let (mut app, send) = setup_test();

        let mesh = 0;
        app.world.spawn(MeshId(mesh));

        let mut meshes = app.world.resource_mut::<Assets<Mesh>>();
        let handle = meshes.add(Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::all(),
        ));

        let id = 1;
        app.world.spawn(WiredPrimitiveBundle {
            id: PrimitiveId(id),
            mesh: MeshId(mesh),
            handle_mesh: handle.clone(),
        });

        let value = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];

        send.send(ScriptAction::SetPrimitiveNormals {
            id,
            value: value.clone(),
        })
        .unwrap();
        app.update();

        let meshes = app.world.resource::<Assets<Mesh>>();
        let mesh = meshes.get(handle).unwrap();

        let attr = mesh
            .attribute(Mesh::ATTRIBUTE_NORMAL)
            .unwrap()
            .as_float3()
            .unwrap()
            .to_vec()
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        assert_eq!(attr, value);
    }

    #[test]
    #[traced_test]
    fn set_primitive_positions() {
        let (mut app, send) = setup_test();

        let mesh = 0;
        app.world.spawn(MeshId(mesh));

        let mut meshes = app.world.resource_mut::<Assets<Mesh>>();
        let handle = meshes.add(Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::all(),
        ));

        let id = 1;
        app.world.spawn(WiredPrimitiveBundle {
            id: PrimitiveId(id),
            mesh: MeshId(mesh),
            handle_mesh: handle.clone(),
        });

        let value = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];

        send.send(ScriptAction::SetPrimitivePositions {
            id,
            value: value.clone(),
        })
        .unwrap();
        app.update();

        let meshes = app.world.resource::<Assets<Mesh>>();
        let mesh = meshes.get(handle).unwrap();

        let attr = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap()
            .to_vec()
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        assert_eq!(attr, value);
    }

    #[test]
    #[traced_test]
    fn set_primitive_uvs() {
        let (mut app, send) = setup_test();

        let mesh = 0;
        app.world.spawn(MeshId(mesh));

        let mut meshes = app.world.resource_mut::<Assets<Mesh>>();
        let handle = meshes.add(Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::all(),
        ));

        let id = 1;
        app.world.spawn(WiredPrimitiveBundle {
            id: PrimitiveId(id),
            mesh: MeshId(mesh),
            handle_mesh: handle.clone(),
        });

        let value = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];

        send.send(ScriptAction::SetPrimitiveUvs {
            id,
            value: value.clone(),
        })
        .unwrap();
        app.update();

        let meshes = app.world.resource::<Assets<Mesh>>();
        let mesh = meshes.get(handle).unwrap();

        // Idk how to read values here, so we just test the length.
        let len = mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap().len();

        assert_eq!(len, value.len() / 2);
    }

    #[test]
    #[traced_test]
    fn set_primitive_material() {
        let (mut app, send) = setup_test();

        let mesh = 0;
        app.world.spawn(MeshId(mesh));

        let mut meshes = app.world.resource_mut::<Assets<Mesh>>();
        let handle_mesh = meshes.add(Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::all(),
        ));

        let primitive = 1;
        let primitive_ent = app
            .world
            .spawn(WiredPrimitiveBundle {
                id: PrimitiveId(primitive),
                mesh: MeshId(mesh),
                handle_mesh: handle_mesh.clone(),
            })
            .id();

        let mut materials = app.world.resource_mut::<Assets<StandardMaterial>>();
        let handle_material = materials.add(StandardMaterial::default());

        let material = 3;
        app.world.spawn(WiredMaterialBundle {
            id: MaterialId(material),
            handle: handle_material.clone(),
        });

        send.send(ScriptAction::SetPrimitiveMaterial {
            id: primitive,
            material: Some(material),
        })
        .unwrap();
        app.update();

        let found_handle = app
            .world
            .get::<Handle<StandardMaterial>>(primitive_ent)
            .unwrap();
        assert_eq!(*found_handle, handle_material);
    }

    // Material
    #[test]
    #[traced_test]
    fn create_material() {
        let (mut app, send) = setup_test();

        let id = 0;
        send.send(ScriptAction::CreateMaterial { id }).unwrap();
        app.update();

        let mut materials = app
            .world
            .query::<(&MaterialId, &Handle<StandardMaterial>)>();
        let (found_id, ..) = materials.single(&app.world);
        assert_eq!(found_id.0, id);
    }

    #[test]
    #[traced_test]
    fn create_material_duplicate_id() {
        let (mut app, send) = setup_test();

        let mut material_assets = app.world.resource_mut::<Assets<StandardMaterial>>();
        let handle = material_assets.add(StandardMaterial::default());

        let id = 0;
        app.world.spawn((MaterialId(id), handle));

        send.send(ScriptAction::CreateMaterial { id }).unwrap();
        app.update();

        let mut materials = app.world.query::<&MaterialId>();
        assert!(materials.iter(&app.world).len() == 1);
    }

    #[test]
    #[traced_test]
    fn remove_material() {
        let (mut app, send) = setup_test();

        let mut material_assets = app.world.resource_mut::<Assets<StandardMaterial>>();
        let handle = material_assets.add(StandardMaterial::default());

        let id = 0;
        let ent = app.world.spawn((MaterialId(id), handle)).id();

        send.send(ScriptAction::RemoveMaterial { id }).unwrap();
        app.update();

        assert!(app.world.get_entity(ent).is_none());
    }

    #[test]
    #[traced_test]
    fn remove_invalid_material() {
        let (mut app, send) = setup_test();

        send.send(ScriptAction::RemoveMaterial { id: 0 }).unwrap();
        app.update();
    }

    #[test]
    #[traced_test]
    fn set_material_color() {
        let (mut app, send) = setup_test();

        let mut material_assets = app.world.resource_mut::<Assets<StandardMaterial>>();
        let handle = material_assets.add(StandardMaterial::default());

        let id = 0;
        app.world.spawn((MaterialId(id), handle.clone()));

        let color = Color::rgba(0.1, 0.2, 0.3, 0.4);

        send.send(ScriptAction::SetMaterialColor { id, color })
            .unwrap();
        app.update();

        let material_assets = app.world.resource::<Assets<StandardMaterial>>();
        let material = material_assets.get(handle).unwrap();
        assert_eq!(material.base_color, color);
    }

    // Integration
    #[test]
    #[traced_test]
    fn add_remove_primitive_node() {
        let (mut app, send) = setup_test();

        let node_id = 0;
        let mesh_id = 1;
        let primitive_id = 2;

        send.send(ScriptAction::CreateNode { id: node_id }).unwrap();
        send.send(ScriptAction::CreateMesh { id: mesh_id }).unwrap();

        send.send(ScriptAction::SetNodeMesh {
            id: node_id,
            mesh: Some(mesh_id),
        })
        .unwrap();

        send.send(ScriptAction::CreatePrimitive {
            id: primitive_id,
            mesh: mesh_id,
        })
        .unwrap();

        app.update();

        let mut nodes = app.world.query::<&NodeMesh>();
        let node_mesh = nodes.single(&app.world);
        assert_eq!(node_mesh.id, Some(mesh_id));
        assert_eq!(node_mesh.primitives.len(), 1);
        assert!(node_mesh.primitives.contains_key(&primitive_id));

        send.send(ScriptAction::RemovePrimitive {
            id: primitive_id,
            mesh: mesh_id,
        })
        .unwrap();

        app.update();

        let mut nodes = app.world.query::<&NodeMesh>();
        let node_mesh = nodes.single(&app.world);
        assert_eq!(node_mesh.id, Some(mesh_id));
        assert!(node_mesh.primitives.is_empty());
    }

    #[test]
    #[traced_test]
    fn update_used_material() {
        let (mut app, send) = setup_test();

        let node_id = 0;
        let mesh_id = 1;
        let primitive_id = 2;
        let material_id = 3;

        send.send(ScriptAction::CreateNode { id: node_id }).unwrap();
        send.send(ScriptAction::CreateMesh { id: mesh_id }).unwrap();
        send.send(ScriptAction::SetNodeMesh {
            id: node_id,
            mesh: Some(mesh_id),
        })
        .unwrap();

        send.send(ScriptAction::CreatePrimitive {
            id: primitive_id,
            mesh: mesh_id,
        })
        .unwrap();
        send.send(ScriptAction::CreateMaterial { id: material_id })
            .unwrap();
        send.send(ScriptAction::SetPrimitiveMaterial {
            id: primitive_id,
            material: Some(material_id),
        })
        .unwrap();

        let color = Color::Rgba {
            red: 1.0,
            green: 0.5,
            blue: 0.78,
            alpha: 0.6,
        };
        send.send(ScriptAction::SetMaterialColor {
            id: material_id,
            color,
        })
        .unwrap();

        app.update();

        let mut nodes = app.world.query::<&NodeMesh>();
        let node_mesh = nodes.single(&app.world);
        assert_eq!(node_mesh.id, Some(mesh_id));
        assert_eq!(node_mesh.primitives.len(), 1);

        let primitive_ent = node_mesh.primitives[&primitive_id];
        let handle = app
            .world
            .get::<Handle<StandardMaterial>>(primitive_ent)
            .unwrap();
        let material_assets = app.world.resource::<Assets<StandardMaterial>>();
        let material = material_assets.get(handle).unwrap();
        assert_eq!(material.base_color, color);
    }
}
