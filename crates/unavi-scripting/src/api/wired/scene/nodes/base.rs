use std::cell::Cell;

use avian3d::prelude::CollisionLayers;
use bevy::prelude::{Transform as BTransform, *};
use unavi_constants::player::layers::LAYER_WORLD;
use wasm_bridge::component::Resource;
use wasm_bridge_wasi::{ResourceTable, ResourceTableError};

use crate::{
    api::{
        utils::{RefCount, RefCountCell, RefResource},
        wired::{
            input::input_handler::InputHandler,
            math::bindings::types::Transform,
            physics::bindings::types::{Collider, RigidBody},
            scene::{bindings::composition::Asset, mesh::MeshRes},
        },
    },
    data::ScriptData,
};

/// Node data used by both the `node` and `asset-node` resources.
#[derive(Default, Debug)]
pub struct NodeRes {
    pub asset: Option<Asset>,
    pub children: Vec<Resource<NodeRes>>,
    pub collider: Option<Resource<Collider>>,
    pub input_handler: Option<Resource<InputHandler>>,
    pub mesh: Option<Resource<MeshRes>>,
    pub name: String,
    pub parent: Option<Resource<NodeRes>>,
    pub rigid_body: Option<Resource<RigidBody>>,
    pub transform: BTransform,
    ref_count: RefCountCell,
}

impl RefCount for NodeRes {
    fn ref_count(&self) -> &Cell<usize> {
        &self.ref_count
    }
}

impl RefResource for NodeRes {}

impl NodeRes {
    pub fn new_res(data: &mut ScriptData) -> wasm_bridge::Result<Resource<Self>> {
        let node = Self::default();
        let table_res = data.table.push(node)?;
        let res = data.clone_res(&table_res)?;
        let rep = res.rep();

        let nodes = data
            .api
            .wired_scene
            .as_ref()
            .unwrap()
            .entities
            .nodes
            .clone();

        data.commands.push(move |world: &mut World| {
            let entity = world.spawn(WiredNodeBundle::new(rep)).id();
            let mut nodes = nodes.write().unwrap();
            nodes.insert(rep, entity);
        });

        Ok(res)
    }

    pub fn global_transform(
        data: &mut ScriptData,
        res: &Resource<Self>,
    ) -> wasm_bridge::Result<Transform> {
        let node = data.table.get(res)?;
        let transform = calc_global_transform(BTransform::default(), node, &data.table)?;
        Ok(transform.into())
    }
    pub fn set_transform(
        data: &mut ScriptData,
        res: &Resource<Self>,
        value: Transform,
    ) -> wasm_bridge::Result<()> {
        let node = data.table.get_mut(res)?;

        node.transform = value.into();

        let mut transform = BTransform {
            translation: value.translation.into(),
            rotation: value.rotation.into(),
            scale: value.scale.into(),
        };

        // parry3d (used in avian physics) panics when scale is 0
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

        data.node_insert(res.rep(), transform);

        Ok(())
    }

    pub fn parent(
        data: &mut ScriptData,
        res: &Resource<Self>,
    ) -> wasm_bridge::Result<Option<Resource<Self>>> {
        let node = data.table.get(res)?;
        let parent = match &node.parent {
            Some(r) => Some(data.clone_res(r)?),
            None => None,
        };
        Ok(parent)
    }
    pub fn children(
        data: &mut ScriptData,
        res: &Resource<Self>,
    ) -> wasm_bridge::Result<Vec<Resource<Self>>> {
        let node = data.table.get(res)?;
        let children = node
            .children
            .iter()
            .map(|res| data.clone_res(res))
            .collect::<Result<_, _>>()?;
        Ok(children)
    }
    pub fn add_child(
        data: &mut ScriptData,
        res: &Resource<Self>,
        value: &Resource<Self>,
    ) -> wasm_bridge::Result<()> {
        let child_rep = value.rep();
        let parent_rep = res.rep();

        // Add child to children.
        let child_res = data.clone_res(value)?;
        let node = data.table.get_mut(res)?;
        node.children.push(child_res);

        // Remove child from old parent's children.
        let child_node = data.table.get(value)?;
        if let Some(parent) = &child_node.parent {
            let parent = data.clone_res(parent)?;
            Self::remove_child(data, &parent, value)?;
        }

        // Set parent.
        let parent_res = data.clone_res(res)?;
        let child = data.table.get_mut(value)?;
        child.parent = Some(parent_res);

        // Update ECS.
        let nodes = data
            .api
            .wired_scene
            .as_ref()
            .unwrap()
            .entities
            .nodes
            .clone();

        data.commands.push(move |world: &mut World| {
            let nodes = nodes.read().unwrap();
            let child_ent = nodes.get(&child_rep).unwrap();
            let parent_ent = nodes.get(&parent_rep).unwrap();

            world.entity_mut(*parent_ent).push_children(&[*child_ent]);
        });

        Ok(())
    }
    pub fn remove_child(
        data: &mut ScriptData,
        res: &Resource<Self>,
        value: &Resource<Self>,
    ) -> wasm_bridge::Result<()> {
        let child_rep = value.rep();

        // Remove from children.
        let node = data.table.get_mut(res)?;
        node.children
            .iter()
            .position(|r| r.rep() == child_rep)
            .map(|index| node.children.remove(index));

        // Remove parent.
        let child_node = data.table.get_mut(value)?;
        child_node.parent = None;

        // Update ECS.
        let nodes = data
            .api
            .wired_scene
            .as_ref()
            .unwrap()
            .entities
            .nodes
            .clone();

        data.commands.push(move |world: &mut World| {
            let nodes = nodes.read().unwrap();
            let child_ent = nodes.get(&child_rep).unwrap();
            world.entity_mut(*child_ent).remove_parent();
        });

        Ok(())
    }
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

fn calc_global_transform(
    transform: BTransform,
    node: &NodeRes,
    table: &ResourceTable,
) -> Result<BTransform, ResourceTableError> {
    let new_transform = node.transform * transform;

    if let Some(parent) = &node.parent {
        let parent = table.get(parent)?;
        calc_global_transform(new_transform, parent, table)
    } else {
        Ok(new_transform)
    }
}
