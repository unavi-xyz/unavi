use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
        render_asset::RenderAssetUsages,
    },
    utils::HashMap,
};

use crate::Owner;

use super::{WiredGltfAction, WiredGltfReceiver};

#[derive(Component, Debug)]
pub struct NodeId(pub u32);

#[derive(Bundle)]
pub struct WiredNodeBundle {
    pub id: NodeId,
    pub owner: Owner,
    pub spatial: SpatialBundle,
    pub primitives: NodePrimitives,
}

impl WiredNodeBundle {
    pub fn new(id: u32, owner: Entity) -> Self {
        Self {
            id: NodeId(id),
            owner: Owner(owner),
            primitives: NodePrimitives::default(),
            spatial: SpatialBundle::default(),
        }
    }
}

#[derive(Component, Default, Deref, DerefMut)]
pub struct NodePrimitives(pub HashMap<u32, Entity>);

#[derive(Component, Debug)]
pub struct MeshId(pub u32);

#[derive(Bundle)]
pub struct WiredMeshBundle {
    pub id: MeshId,
    pub owner: Owner,
}

impl WiredMeshBundle {
    pub fn new(id: u32, owner: Entity) -> Self {
        Self {
            id: MeshId(id),
            owner: Owner(owner),
        }
    }
}

#[derive(Component, Debug)]
pub struct PrimitiveId(pub u32);

#[derive(Bundle)]
pub struct WiredPrimitiveBundle {
    pub id: PrimitiveId,
    pub mesh: MeshId,
    pub mesh_handle: Handle<Mesh>,
    pub owner: Owner,
}

pub fn handle_wired_gltf_actions(
    meshes: Query<(Entity, &MeshId)>,
    mut commands: Commands,
    mut default_material: Local<Option<Handle<StandardMaterial>>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut nodes: Query<(Entity, &NodeId, &mut NodePrimitives)>,
    mut transforms: Query<&mut Transform>,
    primitives: Query<(Entity, &PrimitiveId, &MeshId, &Handle<Mesh>)>,
    scripts: Query<(Entity, &WiredGltfReceiver)>,
) {
    if default_material.is_none() {
        *default_material = Some(material_assets.add(StandardMaterial::default()));
    }

    let default_material = default_material.as_ref().unwrap();

    for (entity, receiver) in scripts.iter() {
        // We only take 1 message per frame, as we must wait for deferred commands to be run before
        // processing subsequent messages. For example, after creating a node we must wait for
        // that entity to be spawned so other messages can query that entity if needed.
        // A better solution would likely be to use the World directly, but we are using a few convenience methods
        // provided by Commands, so for now this is fine.
        if let Ok(msg) = receiver.try_recv() {
            match msg {
                WiredGltfAction::CreateMesh { id } => {
                    let span = info_span!("CreateMesh", id);
                    let s = span.entered();

                    if find_mesh(&meshes, id).is_some() {
                        warn!("Mesh {} already exists.", id);
                    } else {
                        commands.spawn(WiredMeshBundle::new(id, entity));
                    }

                    drop(s);
                }
                WiredGltfAction::CreateNode { id } => {
                    let span = info_span!("CreateNode", id);
                    let s = span.entered();

                    if find_node(&mut nodes, id).is_some() {
                        warn!("Node {} already exists.", id);
                    } else {
                        commands.spawn(WiredNodeBundle::new(id, entity));
                    }

                    drop(s);
                }
                WiredGltfAction::CreatePrimitive { id, mesh } => {
                    let span = info_span!("CreatePrimitive", id, mesh);
                    let s = span.entered();

                    if find_mesh(&meshes, mesh).is_none() {
                        warn!("Mesh {} not found.", mesh);
                    } else if find_primitive(&primitives, id).is_some() {
                        warn!("Primitive {} already exists.", id);
                    } else {
                        commands.spawn((
                            SpatialBundle::default(),
                            WiredPrimitiveBundle {
                                id: PrimitiveId(id),
                                mesh: MeshId(mesh),
                                owner: Owner(entity),
                                mesh_handle: mesh_assets.add(Mesh::new(
                                    PrimitiveTopology::TriangleList,
                                    RenderAssetUsages::all(),
                                )),
                            },
                            default_material.clone(),
                        ));
                    }

                    drop(s);
                }
                WiredGltfAction::RemoveMesh { id } => {
                    let span = info_span!("RemoveMesh", id);
                    let s = span.entered();

                    if let Some(ent) = find_mesh(&meshes, id) {
                        commands.entity(ent).despawn_recursive();
                    } else {
                        warn!("Mesh {} does not exist", id);
                    }

                    drop(s);
                }
                WiredGltfAction::RemoveNode { id } => {
                    let span = info_span!("RemoveNode", id);
                    let s = span.entered();

                    if let Some((ent, ..)) = find_node(&mut nodes, id) {
                        commands.entity(ent).despawn_recursive();
                    } else {
                        warn!("Node {} does not exist", id);
                    }

                    drop(s);
                }
                WiredGltfAction::RemovePrimitive { id } => {
                    let span = info_span!("RemovePrimitive", id);
                    let s = span.entered();

                    if let Some((ent, ..)) = find_primitive(&primitives, id) {
                        commands.entity(ent).despawn_recursive();
                    } else {
                        warn!("Primitive {} does not exist", id);
                    }

                    drop(s);
                }
                WiredGltfAction::SetNodeParent { id, parent } => {
                    let span = info_span!("SetNodeParent", id, parent);
                    let s = span.entered();

                    if let Some((ent, ..)) = find_node(&mut nodes, id) {
                        if let Some(parent) = parent {
                            if let Some((parent_ent, ..)) = find_node(&mut nodes, parent) {
                                commands.entity(parent_ent).push_children(&[ent]);
                            } else {
                                commands.entity(ent).remove_parent();
                            }
                        } else {
                            commands.entity(ent).remove_parent();
                        }
                    } else {
                        warn!("Node {} does not exist", id);
                    }

                    drop(s);
                }
                WiredGltfAction::SetNodeTransform { id, transform } => {
                    let span = info_span!("SetNodeTransform", id);
                    let s = span.entered();

                    if let Some((ent, ..)) = find_node(&mut nodes, id) {
                        let mut node_transform = transforms.get_mut(ent).unwrap();
                        node_transform.clone_from(&transform);
                    }

                    drop(s);
                }
                WiredGltfAction::SetNodeMesh { id, mesh } => {
                    let span = info_span!("SetNodeMesh", id, mesh);
                    let s = span.entered();

                    if let Some((ent, mut node_primitives)) = find_node(&mut nodes, id) {
                        if let Some(mesh) = mesh {
                            let primitives = primitives
                                .iter()
                                .filter_map(|(_, pid, m, handle)| {
                                    if m.0 == mesh {
                                        Some((pid.0, handle))
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<_>>();

                            let mut to_remove = Vec::new();

                            for id in node_primitives.keys() {
                                if !primitives.iter().any(|(pid, _)| pid == id) {
                                    to_remove.push(*id);
                                }
                            }

                            for id in to_remove {
                                let ent = node_primitives.remove(&id).unwrap();
                                commands.entity(ent).despawn();
                            }

                            for (id, handle) in primitives {
                                if node_primitives.contains_key(&id) {
                                    continue;
                                }

                                let primitive_ent = commands.spawn(handle.clone()).id();
                                commands.entity(ent).add_child(primitive_ent);
                                node_primitives.insert(id, primitive_ent);
                            }
                        } else {
                            for ent in node_primitives.values() {
                                commands.entity(*ent).despawn();
                            }

                            node_primitives.clear();
                        }
                    } else {
                        warn!("Node {} does not exist", id);
                    }

                    drop(s);
                }
                WiredGltfAction::SetPrimitiveIndices { id, value } => {
                    let span = info_span!("SetPrimitiveIndices", id);
                    let s = span.entered();

                    if let Some((_, handle)) = find_primitive(&primitives, id) {
                        let mesh = mesh_assets.get_mut(handle).unwrap();
                        mesh.insert_indices(Indices::U32(value));
                    } else {
                        warn!("Primitive {} does not exist", id);
                    }

                    drop(s);
                }
                WiredGltfAction::SetPrimitiveNormals { id, value } => {
                    let span = info_span!("SetPrimitiveNormals", id);
                    let s = span.entered();

                    if let Some((_, handle)) = find_primitive(&primitives, id) {
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
                WiredGltfAction::SetPrimitivePositions { id, value } => {
                    let span = info_span!("SetPrimitivePositions", id);
                    let s = span.entered();

                    if let Some((_, handle)) = find_primitive(&primitives, id) {
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
                WiredGltfAction::SetPrimitiveUvs { id, value } => {
                    let span = info_span!("SetPrimitiveUvs", id);
                    let s = span.entered();

                    if let Some((_, handle)) = find_primitive(&primitives, id) {
                        let mesh = mesh_assets.get_mut(handle).unwrap();

                        let value = value.chunks(2).map(|x| [x[0], x[1]]).collect::<Vec<_>>();

                        mesh.insert_attribute(
                            Mesh::ATTRIBUTE_UV_0,
                            VertexAttributeValues::Float32x2(value),
                        );
                    } else {
                        warn!("Primitive {} does not exist", id);
                    }

                    drop(s);
                }
            };
        }
    }
}

fn find_node<'a>(
    nodes: &'a mut Query<(Entity, &NodeId, &mut NodePrimitives)>,
    id: u32,
) -> Option<(Entity, Mut<'a, NodePrimitives>)> {
    nodes.iter_mut().find_map(|(ent, nid, primitives)| {
        if nid.0 == id {
            Some((ent, primitives))
        } else {
            None
        }
    })
}

fn find_mesh(meshes: &Query<(Entity, &MeshId)>, id: u32) -> Option<Entity> {
    meshes
        .iter()
        .find_map(|(ent, mid)| if mid.0 == id { Some(ent) } else { None })
}

fn find_primitive<'a>(
    primitives: &'a Query<(Entity, &PrimitiveId, &MeshId, &Handle<Mesh>)>,
    id: u32,
) -> Option<(Entity, &'a Handle<Mesh>)> {
    primitives.iter().find_map(|(ent, pid, _, handle)| {
        if pid.0 == id {
            Some((ent, handle))
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use crossbeam::channel::Sender;

    use super::*;

    fn setup_test() -> (App, Sender<WiredGltfAction>) {
        let mut app = App::new();

        app.add_plugins(AssetPlugin::default())
            .init_asset::<Mesh>()
            .init_asset::<StandardMaterial>()
            .add_systems(Update, handle_wired_gltf_actions);

        let (send, recv) = crossbeam::channel::unbounded();
        app.world.spawn(WiredGltfReceiver(recv));

        (app, send)
    }

    // Node
    #[test]
    fn create_node() {
        let (mut app, send) = setup_test();

        let id = 0;
        send.send(WiredGltfAction::CreateNode { id }).unwrap();
        app.update();

        let mut nodes = app.world.query::<&NodeId>();
        let found_id = nodes.single(&app.world);
        assert_eq!(found_id.0, id);
    }

    #[test]
    fn create_node_duplicate_id() {
        let (mut app, send) = setup_test();

        let owner = app.world.spawn(()).id();

        let id = 0;
        app.world.spawn(WiredNodeBundle::new(id, owner));

        send.send(WiredGltfAction::CreateNode { id }).unwrap();
        app.update();

        let mut nodes = app.world.query::<&NodeId>();
        assert!(nodes.iter(&app.world).len() == 1);
    }

    #[test]
    fn remove_node() {
        let (mut app, send) = setup_test();

        let owner = app.world.spawn(()).id();

        let id = 0;
        let ent = app.world.spawn(WiredNodeBundle::new(id, owner)).id();

        send.send(WiredGltfAction::RemoveNode { id }).unwrap();
        app.update();

        assert!(app.world.get_entity(ent).is_none());
    }

    #[test]
    fn remove_invalid_node() {
        let (mut app, send) = setup_test();

        send.send(WiredGltfAction::RemoveNode { id: 0 }).unwrap();
        app.update();
    }

    #[test]
    fn set_node_parent_some() {
        let (mut app, send) = setup_test();

        let owner = app.world.spawn(()).id();

        let parent_id = 0;
        let parent_ent = app.world.spawn(WiredNodeBundle::new(parent_id, owner)).id();

        let child_id = 1;
        let child_ent = app.world.spawn(WiredNodeBundle::new(child_id, owner)).id();

        send.send(WiredGltfAction::SetNodeParent {
            id: child_id,
            parent: Some(parent_id),
        })
        .unwrap();
        app.update();

        let children = app.world.get::<Children>(parent_ent).unwrap();
        assert!(children.contains(&child_ent));
    }

    #[test]
    fn set_node_parent_none() {
        let (mut app, send) = setup_test();

        let owner = app.world.spawn(()).id();

        let parent_id = 0;
        let parent_ent = app.world.spawn(WiredNodeBundle::new(parent_id, owner)).id();

        let child_id = 1;
        let mut child_ent = None;
        app.world.entity_mut(parent_ent).with_children(|builder| {
            child_ent = Some(builder.spawn(WiredNodeBundle::new(child_id, owner)).id());
        });
        let child_ent = child_ent.unwrap();

        let children = app.world.get::<Children>(parent_ent).unwrap();
        assert!(children.contains(&child_ent));

        send.send(WiredGltfAction::SetNodeParent {
            id: child_id,
            parent: None,
        })
        .unwrap();
        app.update();

        assert!(app.world.get::<Children>(parent_ent).is_none());
    }

    #[test]
    fn set_node_invalid_parent() {
        let (mut app, send) = setup_test();

        let owner = app.world.spawn(()).id();

        let parent_id = 0;
        let child_id = 1;
        app.world.spawn(WiredNodeBundle::new(child_id, owner));

        send.send(WiredGltfAction::SetNodeParent {
            id: child_id,
            parent: Some(parent_id),
        })
        .unwrap();
        app.update();
    }

    #[test]
    fn set_node_transform() {
        let (mut app, send) = setup_test();

        let owner = app.world.spawn(()).id();

        let id = 0;
        let ent = app.world.spawn(WiredNodeBundle::new(id, owner)).id();

        let transform = Transform {
            translation: Vec3::splat(1.0),
            rotation: Quat::from_xyzw(0.1, 0.2, 0.3, 0.4),
            scale: Vec3::splat(3.0),
        };

        send.send(WiredGltfAction::SetNodeTransform { id, transform })
            .unwrap();
        app.update();

        assert_eq!(app.world.get::<Transform>(ent).unwrap(), &transform);
    }

    #[test]
    fn set_node_mesh_some() {
        let (mut app, send) = setup_test();

        let owner = app.world.spawn(()).id();

        let id = 0;
        let ent = app.world.spawn(WiredNodeBundle::new(id, owner)).id();

        let mesh_id = 1;
        app.world.spawn(WiredMeshBundle::new(mesh_id, owner));

        let handle = Handle::default();
        let primitive_id = 2;
        app.world.spawn(WiredPrimitiveBundle {
            mesh_handle: handle.clone(),
            id: PrimitiveId(primitive_id),
            mesh: MeshId(mesh_id),
            owner: Owner(owner),
        });

        send.send(WiredGltfAction::SetNodeMesh {
            id,
            mesh: Some(mesh_id),
        })
        .unwrap();
        app.update();

        let node_primitives = app.world.get::<NodePrimitives>(ent).unwrap();
        let primitive_ent = node_primitives[&primitive_id];

        let found_handle = app.world.get::<Handle<Mesh>>(primitive_ent).unwrap();
        assert_eq!(*found_handle, handle);
    }

    #[test]
    fn set_node_mesh_none() {
        let (mut app, send) = setup_test();

        let owner = app.world.spawn(()).id();

        let mesh_id = 1;
        app.world.spawn(WiredMeshBundle::new(mesh_id, owner));

        let handle = Handle::default();
        let primitive_id = 2;
        app.world.spawn(WiredPrimitiveBundle {
            mesh_handle: handle.clone(),
            id: PrimitiveId(primitive_id),
            mesh: MeshId(mesh_id),
            owner: Owner(owner),
        });

        let primitive_ent = app.world.spawn(handle).id();

        let mut primitives = NodePrimitives::default();
        primitives.insert(primitive_id, primitive_ent);

        let id = 0;
        let ent = app
            .world
            .spawn(WiredNodeBundle {
                id: NodeId(id),
                owner: Owner(owner),
                spatial: SpatialBundle::default(),
                primitives,
            })
            .id();

        send.send(WiredGltfAction::SetNodeMesh { id, mesh: None })
            .unwrap();
        app.update();

        let node_primitives = app.world.get::<NodePrimitives>(ent).unwrap();
        assert!(node_primitives.is_empty());

        assert!(app.world.get_entity(primitive_ent).is_none());
    }

    // Mesh
    #[test]
    fn create_mesh() {
        let (mut app, send) = setup_test();

        let id = 0;
        send.send(WiredGltfAction::CreateMesh { id }).unwrap();
        app.update();

        let mut meshes = app.world.query::<&MeshId>();
        let found_id = meshes.single(&app.world);
        assert_eq!(found_id.0, id);
    }

    #[test]
    fn create_mesh_duplicate_id() {
        let (mut app, send) = setup_test();

        let id = 0;
        app.world.spawn(MeshId(id));

        send.send(WiredGltfAction::CreateMesh { id }).unwrap();
        app.update();

        let mut meshes = app.world.query::<&MeshId>();
        assert!(meshes.iter(&app.world).len() == 1);
    }

    #[test]
    fn remove_mesh() {
        let (mut app, send) = setup_test();

        let id = 0;
        let ent = app.world.spawn(MeshId(id)).id();

        send.send(WiredGltfAction::RemoveMesh { id }).unwrap();
        app.update();

        assert!(app.world.get_entity(ent).is_none());
    }

    #[test]
    fn remove_invalid_mesh() {
        let (mut app, send) = setup_test();

        send.send(WiredGltfAction::RemoveMesh { id: 0 }).unwrap();
        app.update();
    }

    // Primitive
    #[test]
    fn create_primitive() {
        let (mut app, send) = setup_test();

        let mesh = 0;
        app.world.spawn(MeshId(mesh));

        let id = 1;
        send.send(WiredGltfAction::CreatePrimitive { id, mesh })
            .unwrap();
        app.update();

        let mut primitives = app.world.query::<&PrimitiveId>();
        let found_id = primitives.single(&app.world);
        assert_eq!(found_id.0, id);
    }

    #[test]
    fn create_primitive_invalid_mesh() {
        let (mut app, send) = setup_test();

        let mesh = 0;
        let id = 1;
        send.send(WiredGltfAction::CreatePrimitive { id, mesh })
            .unwrap();
        app.update();

        let mut primitives = app.world.query::<&PrimitiveId>();
        assert_eq!(primitives.iter(&app.world).len(), 0);
    }

    #[test]
    fn create_primitive_duplicate_id() {
        let (mut app, send) = setup_test();

        let owner = app.world.spawn(()).id();

        let mesh = 0;
        app.world.spawn(MeshId(mesh));

        let id = 1;
        app.world.spawn(WiredPrimitiveBundle {
            id: PrimitiveId(id),
            mesh: MeshId(mesh),
            mesh_handle: Default::default(),
            owner: Owner(owner),
        });

        send.send(WiredGltfAction::CreatePrimitive { id, mesh })
            .unwrap();
        app.update();

        let mut primitives = app.world.query::<&PrimitiveId>();
        assert!(primitives.iter(&app.world).len() == 1);
    }

    #[test]
    fn remove_primitive() {
        let (mut app, send) = setup_test();

        let owner = app.world.spawn(()).id();

        let mesh = 0;
        app.world.spawn(MeshId(mesh));

        let id = 1;
        let ent = app
            .world
            .spawn(WiredPrimitiveBundle {
                id: PrimitiveId(id),
                mesh: MeshId(mesh),
                mesh_handle: Default::default(),
                owner: Owner(owner),
            })
            .id();

        send.send(WiredGltfAction::RemovePrimitive { id }).unwrap();
        app.update();

        assert!(app.world.get_entity(ent).is_none());
    }

    #[test]
    fn remove_invalid_primitive() {
        let (mut app, send) = setup_test();

        send.send(WiredGltfAction::RemovePrimitive { id: 0 })
            .unwrap();
        app.update();
    }

    #[test]
    fn set_primitive_indices() {
        let (mut app, send) = setup_test();

        let owner = app.world.spawn(()).id();

        let mesh = 0;
        app.world.spawn(MeshId(mesh));

        let mut meshes = app.world.get_resource_mut::<Assets<Mesh>>().unwrap();
        let handle = meshes.add(Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::all(),
        ));

        let id = 1;
        app.world.spawn(WiredPrimitiveBundle {
            id: PrimitiveId(id),
            mesh: MeshId(mesh),
            mesh_handle: handle.clone(),
            owner: Owner(owner),
        });

        let value = vec![0, 1, 2, 3, 4, 5];

        send.send(WiredGltfAction::SetPrimitiveIndices {
            id,
            value: value.clone(),
        })
        .unwrap();
        app.update();

        let meshes = app.world.get_resource::<Assets<Mesh>>().unwrap();
        let mesh = meshes.get(handle).unwrap();

        let indices = match mesh.indices().unwrap() {
            Indices::U32(v) => v,
            _ => panic!(),
        };

        assert_eq!(indices.as_slice(), value.as_slice());
    }

    #[test]
    fn set_primitive_normals() {
        let (mut app, send) = setup_test();

        let owner = app.world.spawn(()).id();

        let mesh = 0;
        app.world.spawn(MeshId(mesh));

        let mut meshes = app.world.get_resource_mut::<Assets<Mesh>>().unwrap();
        let handle = meshes.add(Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::all(),
        ));

        let id = 1;
        app.world.spawn(WiredPrimitiveBundle {
            id: PrimitiveId(id),
            mesh: MeshId(mesh),
            mesh_handle: handle.clone(),
            owner: Owner(owner),
        });

        let value = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];

        send.send(WiredGltfAction::SetPrimitiveNormals {
            id,
            value: value.clone(),
        })
        .unwrap();
        app.update();

        let meshes = app.world.get_resource::<Assets<Mesh>>().unwrap();
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
    fn set_primitive_positions() {
        let (mut app, send) = setup_test();

        let owner = app.world.spawn(()).id();

        let mesh = 0;
        app.world.spawn(MeshId(mesh));

        let mut meshes = app.world.get_resource_mut::<Assets<Mesh>>().unwrap();
        let handle = meshes.add(Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::all(),
        ));

        let id = 1;
        app.world.spawn(WiredPrimitiveBundle {
            id: PrimitiveId(id),
            mesh: MeshId(mesh),
            mesh_handle: handle.clone(),
            owner: Owner(owner),
        });

        let value = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];

        send.send(WiredGltfAction::SetPrimitivePositions {
            id,
            value: value.clone(),
        })
        .unwrap();
        app.update();

        let meshes = app.world.get_resource::<Assets<Mesh>>().unwrap();
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
    fn set_primitive_uvs() {
        let (mut app, send) = setup_test();

        let owner = app.world.spawn(()).id();

        let mesh = 0;
        app.world.spawn(MeshId(mesh));

        let mut meshes = app.world.get_resource_mut::<Assets<Mesh>>().unwrap();
        let handle = meshes.add(Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::all(),
        ));

        let id = 1;
        app.world.spawn(WiredPrimitiveBundle {
            id: PrimitiveId(id),
            mesh: MeshId(mesh),
            mesh_handle: handle.clone(),
            owner: Owner(owner),
        });

        let value = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];

        send.send(WiredGltfAction::SetPrimitiveUvs {
            id,
            value: value.clone(),
        })
        .unwrap();
        app.update();

        let meshes = app.world.get_resource::<Assets<Mesh>>().unwrap();
        let mesh = meshes.get(handle).unwrap();

        // Idk how to read values here, so we just test the length.
        let len = mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap().len();

        assert_eq!(len, value.len() / 2);
    }
}
