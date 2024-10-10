use std::sync::{Arc, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

use avian3d::prelude::CollisionLayers;
use bevy::prelude::{Transform as BTransform, *};
use unavi_constants::player::layers::LAYER_WORLD;
use wasm_bridge::component::Resource;

use crate::{
    api::{
        id::ResourceId,
        wired::{
            input::input_handler::InputHandlerRes,
            math::bindings::types::Transform,
            physics::{collider::ColliderRes, rigid_body::RigidBodyRes},
            scene::{composition::CompositionRes, document::DocumentRes, mesh::MeshRes},
        },
    },
    data::{CommandSender, ControlSender, ScriptControl, ScriptData},
};

#[derive(Debug, Clone)]
pub struct NodeRes {
    command_send: CommandSender,
    control_send: ControlSender,
    data: Arc<RwLock<NodeData>>,
}

/// Node data used by both the `node` and `asset-node` resources.
#[derive(Default, Debug)]
pub struct NodeData {
    pub id: ResourceId,
    pub asset: Option<AssetData>,
    pub children: Vec<NodeRes>,
    pub collider: Option<ColliderRes>,
    pub entity: OnceLock<Entity>,
    pub input_handler: Option<InputHandlerRes>,
    pub mesh: Option<MeshRes>,
    pub name: String,
    pub parent: Option<NodeRes>,
    pub rigid_body: Option<RigidBodyRes>,
    pub transform: BTransform,
}

#[derive(Clone, Debug)]
pub enum AssetData {
    Composition(CompositionRes),
    Document(DocumentRes),
}

#[derive(Component, Clone, Copy, Debug)]
pub struct NodeId(pub u32);

#[derive(Bundle)]
pub struct WiredNodeBundle {
    pub id: NodeId,
    pub layers: CollisionLayers,
    pub spatial: SpatialBundle,
}

impl WiredNodeBundle {
    pub fn new(id: u32) -> Self {
        Self {
            id: NodeId(id),
            layers: CollisionLayers {
                memberships: LAYER_WORLD,
                filters: LAYER_WORLD,
            },
            spatial: SpatialBundle::default(),
        }
    }
}

impl NodeRes {
    pub fn read(&self) -> RwLockReadGuard<NodeData> {
        self.data.read().unwrap()
    }
    pub fn write(&self) -> RwLockWriteGuard<NodeData> {
        self.data.write().unwrap()
    }
    pub fn raw_data(&self) -> &Arc<RwLock<NodeData>> {
        &self.data
    }

    pub fn new(data: &mut ScriptData) -> Self {
        let node = Self {
            command_send: data.command_send.clone(),
            control_send: data.control_send.clone(),
            data: Arc::new(RwLock::new(NodeData::default())),
        };

        {
            let node = node.clone();
            data.command_send
                .try_send(Box::new(move |world: &mut World| {
                    let node_data = node.write();
                    let entity = world.spawn(WiredNodeBundle::new(node_data.id.into())).id();
                    node_data.entity.set(entity).unwrap();
                }))
                .unwrap();
        }

        node
    }

    pub fn new_res(data: &mut ScriptData) -> wasm_bridge::Result<Resource<Self>> {
        let node = Self::new(data);
        let res = data.table.push(node)?;
        Ok(res)
    }

    pub fn global_transform(
        self_: &mut ScriptData,
        res: &Resource<Self>,
    ) -> wasm_bridge::Result<Transform> {
        let node = self_.table.get(res)?;
        let transform = calc_global_transform(BTransform::default(), node);
        Ok(transform.into())
    }
    pub fn set_transform(
        self_: &mut ScriptData,
        res: &Resource<Self>,
        value: Transform,
    ) -> wasm_bridge::Result<()> {
        let mut data = self_.table.get_mut(res)?.write();
        data.transform = value.into();

        let mut transform = BTransform {
            translation: value.translation.into(),
            rotation: value.rotation.into(),
            scale: value.scale.into(),
        };

        // parry3d (used in avian physics) panics when scale is 0
        // TODO: better solution for this, enforce it in the ECS every frame?
        const ALMOST_ZERO: f32 = 10e-10;
        if transform.scale.x == 0.0 {
            transform.scale.x = ALMOST_ZERO;
        }
        if transform.scale.y == 0.0 {
            transform.scale.y = ALMOST_ZERO;
        }
        if transform.scale.z == 0.0 {
            transform.scale.z = ALMOST_ZERO;
        }

        // self_.node_insert(res.rep(), transform);

        Ok(())
    }

    pub fn parent(
        data: &mut ScriptData,
        node: Self,
    ) -> wasm_bridge::Result<Option<Resource<Self>>> {
        let parent = node.read().parent.clone();
        let res = match parent {
            Some(d) => Some(data.table.push(d)?),
            None => None,
        };
        Ok(res)
    }
    pub fn children(data: &mut ScriptData, node: Self) -> wasm_bridge::Result<Vec<Resource<Self>>> {
        let children = node.read().children.clone();
        let res = children
            .into_iter()
            .map(|n| data.table.push(n))
            .collect::<Result<_, _>>()?;
        Ok(res)
    }
    pub fn add_child(data: &mut ScriptData, parent: Self, child: Self) {
        if parent.read().id == child.read().id {
            return;
        }

        // Add to children.
        parent.write().children.push(child.clone());

        // Remove child from old parent's children.
        if let Some(parent) = &child.read().parent {
            Self::remove_child(data, parent.clone(), child.clone());
        }

        // Set new parent.
        child.write().parent = Some(parent.clone());

        // Update ECS.
        data.command_send
            .try_send(Box::new(move |world: &mut World| {
                let parent_ent = *parent.read().entity.get().unwrap();
                let child_ent = *child.read().entity.get().unwrap();

                world.entity_mut(parent_ent).push_children(&[child_ent]);
            }))
            .unwrap();
    }
    pub fn remove_child(data: &mut ScriptData, parent: Self, child: Self) {
        // Remove from children.
        let mut parent_write = parent.write();
        let child_id = child.read().id;

        let found = parent_write
            .children
            .iter()
            .position(|c| c.read().id == child_id)
            .map(|index| parent_write.children.remove(index));

        if found.is_none() {
            return;
        }

        // Remove parent.
        child.write().parent = None;

        // Update ECS.
        data.command_send
            .try_send(Box::new(move |world: &mut World| {
                let child_ent = *child.read().entity.get().unwrap();
                world.entity_mut(child_ent).remove_parent();
            }))
            .unwrap();
    }
}

fn calc_global_transform(transform: BTransform, node: &NodeRes) -> BTransform {
    let data = node.read();
    let new_transform = data.transform * transform;

    if let Some(parent) = &data.parent {
        calc_global_transform(new_transform, parent)
    } else {
        new_transform
    }
}

impl Drop for NodeRes {
    fn drop(&mut self) {
        // Only cleanup when this is the last reference.
        if Arc::strong_count(&self.data) == 1 {
            let data = self.data.clone();

            if let Err(e) = self
                .command_send
                .try_send(Box::new(move |world: &mut World| {
                    let entity = *data.read().unwrap().entity.get().unwrap();
                    world.entity_mut(entity).despawn();
                }))
            {
                // Should only error when the entire script environment is being
                // dropped, in which case we do not matter.
                let _ = self.control_send.send(ScriptControl::Exit(e.to_string()));
            }
        }
    }
}
