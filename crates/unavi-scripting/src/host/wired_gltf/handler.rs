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
                    commands.spawn((Ownership(entity), NodeId(id), SpatialBundle::default()));
                }
                WiredGltfAction::RemoveNode { id } => {
                    if let Some((ent, ..)) = nodes.iter().find(|(_, nid)| nid.0 == id) {
                        commands.entity(ent).despawn_recursive();
                    }
                }
                WiredGltfAction::SetParent { id, parent } => {
                    if let Some((ent, ..)) = nodes.iter().find(|(_, nid)| nid.0 == id) {
                        match parent {
                            Some(parent) => match nodes.iter().find(|(_, nid)| nid.0 == parent) {
                                Some((ent_parent, ..)) => {
                                    commands.entity(ent_parent).push_children(&[ent]);
                                }
                                None => {
                                    commands.entity(ent).remove_parent();
                                }
                            },
                            None => {
                                commands.entity(ent).remove_parent();
                            }
                        };
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crossbeam::channel::Sender;

    use super::*;

    // TODO: More testing, this is input from untrusted user scripts.
    // Test when values are unexpected - duplicate IDs, missing IDs, etc.

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
    fn remove_node() {
        let (mut app, send) = setup_test();

        let id = 0;
        let ent = app.world.spawn(NodeId(id)).id();

        send.send(WiredGltfAction::RemoveNode { id }).unwrap();
        app.update();

        assert!(app.world.get_entity(ent).is_none());
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
}
