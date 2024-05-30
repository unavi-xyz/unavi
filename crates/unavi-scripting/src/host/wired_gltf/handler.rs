use bevy::prelude::*;

use crate::Ownership;

use super::{WiredGltfAction, WiredGltfReceiver};

#[derive(Component, Debug)]
pub struct NodeId(u32);

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
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: More testing, this is input from untrusted user scripts.
    // Test when values are unexpected - duplicate IDs, missing IDs, etc.

    #[test]
    fn test_create_remove_node() {
        let mut app = App::new();
        app.add_systems(Update, handle_wired_gltf_actions);

        let (send, recv) = crossbeam::channel::unbounded();
        app.world.spawn(WiredGltfReceiver(recv));

        let id = 0;
        send.send(WiredGltfAction::CreateNode { id }).unwrap();
        app.update();

        let mut nodes = app.world.query::<(Entity, &NodeId)>();
        let (node_entity, node_id) = nodes.single(&app.world);
        assert_eq!(node_id.0, id);

        send.send(WiredGltfAction::RemoveNode { id }).unwrap();
        app.update();

        assert_eq!(nodes.iter(&app.world).count(), 0);
        assert!(app.world.get_entity(node_entity).is_none());
    }
}
