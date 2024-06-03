use bevy::prelude::*;

use crate::Ownership;

use super::{WiredGltfAction, WiredGltfReceiver};

#[derive(Component, Debug)]
pub struct NodeId(pub u32);

pub fn handle_wired_gltf_actions(
    mut commands: Commands,
    nodes: Query<(Entity, &NodeId)>,
    scripts: Query<(Entity, &WiredGltfReceiver)>,
) {
    for (entity, receiver) in scripts.iter() {
        while let Ok(msg) = receiver.try_recv() {
            match msg {
                WiredGltfAction::CreateNode { id } => {
                    if find_node(&nodes, id).is_some() {
                        warn!("Node {} already exists.", id);
                    } else {
                        commands.spawn((Ownership(entity), NodeId(id), SpatialBundle::default()));
                    }
                }
                WiredGltfAction::RemoveNode { id } => {
                    if let Some(ent) = find_node(&nodes, id) {
                        commands.entity(ent).despawn_recursive();
                    } else {
                        warn!("Node {} does not exist", id);
                    }
                }
                WiredGltfAction::SetParent { id, parent } => {
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
            }
        }
    }
}

fn find_node(nodes: &Query<(Entity, &NodeId)>, id: u32) -> Option<Entity> {
    nodes
        .iter()
        .find_map(|(ent, nid)| if nid.0 == id { Some(ent) } else { None })
}

#[cfg(test)]
mod tests {
    use crossbeam::channel::Sender;

    use super::*;

    fn setup_test() -> (App, Sender<WiredGltfAction>) {
        let mut app = App::new();
        app.add_systems(Update, handle_wired_gltf_actions);

        let (send, recv) = crossbeam::channel::unbounded();
        app.world.spawn(WiredGltfReceiver(recv));

        (app, send)
    }

    #[test]
    fn create_node() {
        let (mut app, send) = setup_test();

        let id = 0;
        send.send(WiredGltfAction::CreateNode { id }).unwrap();
        app.update();

        let mut nodes = app.world.query::<&NodeId>();
        let node_id = nodes.single(&app.world);
        assert_eq!(node_id.0, id);
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

        send.send(WiredGltfAction::SetParent {
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

        send.send(WiredGltfAction::SetParent {
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

        send.send(WiredGltfAction::SetParent {
            id: child_id,
            parent: Some(parent_id),
        })
        .unwrap();
        app.update();
    }
}
