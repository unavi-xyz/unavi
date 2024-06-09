use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
        render_asset::RenderAssetUsages,
    },
};

use crate::Owner;

use super::{WiredGltfAction, WiredGltfReceiver};

#[derive(Component, Debug)]
pub struct NodeId(pub u32);

#[derive(Bundle)]
pub struct WiredNodeBundle {
    id: NodeId,
    owner: Owner,
    spatial: SpatialBundle,
}

#[derive(Component, Debug)]
pub struct MeshId(pub u32);

#[derive(Bundle)]
pub struct WiredMeshBundle {
    id: MeshId,
    owner: Owner,
}

#[derive(Component, Debug)]
pub struct PrimitiveId(pub u32);

#[derive(Bundle)]
pub struct WiredPrimitiveBundle {
    handle: Handle<Mesh>,
    id: PrimitiveId,
    mesh: MeshId,
    owner: Owner,
}

pub fn handle_wired_gltf_actions(
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut transforms: Query<&mut Transform>,
    scripts: Query<(Entity, &WiredGltfReceiver)>,
    nodes: Query<(Entity, &NodeId)>,
    meshes: Query<(Entity, &MeshId)>,
    primitives: Query<(Entity, &PrimitiveId, &Handle<Mesh>)>,
) {
    for (entity, receiver) in scripts.iter() {
        while let Ok(msg) = receiver.try_recv() {
            match msg {
                WiredGltfAction::CreateMesh { id } => {
                    if find_mesh(&meshes, id).is_some() {
                        warn!("Mesh {} already exists.", id);
                    } else {
                        commands.spawn(WiredMeshBundle {
                            id: MeshId(id),
                            owner: Owner(entity),
                        });
                    }
                }
                WiredGltfAction::CreateNode { id } => {
                    if find_node(&nodes, id).is_some() {
                        warn!("Node {} already exists.", id);
                    } else {
                        commands.spawn(WiredNodeBundle {
                            id: NodeId(id),
                            owner: Owner(entity),
                            spatial: SpatialBundle::default(),
                        });
                    }
                }
                WiredGltfAction::CreatePrimitive { id, mesh } => {
                    if find_mesh(&meshes, mesh).is_none() {
                        warn!("Mesh {} node found.", id);
                    } else if find_primitive(&primitives, id).is_some() {
                        warn!("Primitive {} already exists.", id);
                    } else {
                        commands.spawn(WiredPrimitiveBundle {
                            id: PrimitiveId(id),
                            mesh: MeshId(mesh),
                            owner: Owner(entity),
                            handle: mesh_assets.add(Mesh::new(
                                PrimitiveTopology::TriangleList,
                                RenderAssetUsages::all(),
                            )),
                        });
                    }
                }
                WiredGltfAction::RemoveMesh { id } => {
                    if let Some(ent) = find_mesh(&meshes, id) {
                        commands.entity(ent).despawn_recursive();
                    } else {
                        warn!("Mesh {} does not exist", id);
                    }
                }
                WiredGltfAction::RemoveNode { id } => {
                    if let Some(ent) = find_node(&nodes, id) {
                        commands.entity(ent).despawn_recursive();
                    } else {
                        warn!("Node {} does not exist", id);
                    }
                }
                WiredGltfAction::RemovePrimitive { id } => {
                    if let Some((ent, ..)) = find_primitive(&primitives, id) {
                        commands.entity(ent).despawn_recursive();
                    } else {
                        warn!("Primitive {} does not exist", id);
                    }
                }
                WiredGltfAction::SetNodeParent { id, parent } => {
                    if let Some(ent) = find_node(&nodes, id) {
                        if let Some(parent) = parent {
                            if let Some(parent_ent) = find_node(&nodes, parent) {
                                commands.entity(parent_ent).push_children(&[ent]);
                            } else {
                                commands.entity(ent).remove_parent();
                            }
                        } else {
                            commands.entity(ent).remove_parent();
                        }
                    }
                }
                WiredGltfAction::SetNodeTransform { id, transform } => {
                    if let Some(ent) = find_node(&nodes, id) {
                        let mut node_transform = transforms.get_mut(ent).unwrap();
                        node_transform.clone_from(&transform);
                    }
                }
                WiredGltfAction::SetPrimitiveIndices { id, value } => {
                    if let Some((_, handle)) = find_primitive(&primitives, id) {
                        let mesh = mesh_assets.get_mut(handle).unwrap();
                        mesh.insert_indices(Indices::U32(value));
                    } else {
                        warn!("Primitive {} does not exist", id);
                    }
                }
                WiredGltfAction::SetPrimitiveNormals { id, value } => {
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
                }
                WiredGltfAction::SetPrimitivePositions { id, value } => {
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
                }
                WiredGltfAction::SetPrimitiveUvs { id, value } => {
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
                }
            }
        }
    }
}

fn find_node(nodes: &Query<(Entity, &NodeId)>, id: u32) -> Option<Entity> {
    nodes
        .iter()
        .find_map(|(ent, nid)| if nid.0 == id { Some(ent) } else { None })
}

fn find_mesh(meshes: &Query<(Entity, &MeshId)>, id: u32) -> Option<Entity> {
    meshes
        .iter()
        .find_map(|(ent, mid)| if mid.0 == id { Some(ent) } else { None })
}

fn find_primitive(
    primitives: &Query<(Entity, &PrimitiveId, &Handle<Mesh>)>,
    id: u32,
) -> Option<(Entity, Handle<Mesh>)> {
    primitives.iter().find_map(|(ent, pid, handle)| {
        if pid.0 == id {
            Some((ent, handle.clone()))
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

        let id = 0;
        app.world.spawn(NodeId(id));

        send.send(WiredGltfAction::CreateNode { id }).unwrap();
        app.update();

        let mut nodes = app.world.query::<&NodeId>();
        assert!(nodes.iter(&app.world).len() == 1);
    }

    #[test]
    fn remove_node() {
        let (mut app, send) = setup_test();

        let id = 0;
        let ent = app.world.spawn(NodeId(id)).id();

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
    fn set_parent_some() {
        let (mut app, send) = setup_test();

        let parent_id = 0;
        let parent_ent = app.world.spawn(NodeId(parent_id)).id();

        let child_id = 1;
        let child_ent = app.world.spawn(NodeId(child_id)).id();

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
    fn set_parent_none() {
        let (mut app, send) = setup_test();

        let parent_id = 0;
        let parent_ent = app.world.spawn(NodeId(parent_id)).id();

        let child_id = 1;
        let mut child_ent = None;
        app.world.entity_mut(parent_ent).with_children(|builder| {
            child_ent = Some(builder.spawn(NodeId(child_id)).id());
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
    fn set_invalid_parent() {
        let (mut app, send) = setup_test();

        let parent_id = 0;
        let child_id = 1;
        app.world.spawn(NodeId(child_id));

        send.send(WiredGltfAction::SetNodeParent {
            id: child_id,
            parent: Some(parent_id),
        })
        .unwrap();
        app.update();
    }

    #[test]
    fn set_transform() {
        let (mut app, send) = setup_test();

        let id = 0;
        let ent = app.world.spawn((NodeId(id), SpatialBundle::default())).id();

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
            handle: Default::default(),
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
                handle: Default::default(),
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
            handle: handle.clone(),
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
            handle: handle.clone(),
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
            handle: handle.clone(),
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
            handle: handle.clone(),
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
