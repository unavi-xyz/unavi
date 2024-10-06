use std::sync::{Arc, OnceLock, RwLock};

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
    data::ScriptData,
};

#[derive(Debug, Clone)]
pub struct NodeRes(pub Arc<RwLock<NodeData>>);

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
    pub fn new(data: &mut ScriptData) -> Self {
        let node = Self(Arc::new(RwLock::new(NodeData::default())));

        {
            let node = node.clone();
            data.commands.push(move |world: &mut World| {
                let entity = world
                    .spawn(WiredNodeBundle::new(node.0.read().unwrap().id.into()))
                    .id();
                node.0.write().unwrap().entity.get_or_init(|| entity);
            });
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
        let mut data = self_.table.get_mut(res)?.0.write().unwrap();
        data.transform = value.into();

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

        // self_.node_insert(res.rep(), transform);

        Ok(())
    }

    pub fn parent(
        data: &mut ScriptData,
        node: Self,
    ) -> wasm_bridge::Result<Option<Resource<Self>>> {
        let parent = node.0.read().unwrap().parent.clone();
        let res = match parent {
            Some(d) => Some(data.table.push(d)?),
            None => None,
        };
        Ok(res)
    }
    pub fn children(data: &mut ScriptData, node: Self) -> wasm_bridge::Result<Vec<Resource<Self>>> {
        let children = node.0.read().unwrap().children.clone();
        let res = children
            .into_iter()
            .map(|n| data.table.push(n))
            .collect::<Result<_, _>>()?;
        Ok(res)
    }
    pub fn add_child(data: &mut ScriptData, parent: Self, child: Self) {
        // Add to children.
        parent.0.write().unwrap().children.push(child.clone());

        // Remove child from old parent's children.
        if let Some(parent) = &child.0.read().unwrap().parent {
            Self::remove_child(data, parent.clone(), child.clone());
        }

        // Set new parent.
        child.0.write().unwrap().parent = Some(parent.clone());

        // Update ECS.
        data.commands.push(move |world: &mut World| {
            let parent_ent = *parent.0.read().unwrap().entity.get().unwrap();
            let child_ent = *child.0.read().unwrap().entity.get().unwrap();

            world.entity_mut(parent_ent).push_children(&[child_ent]);
        });
    }
    pub fn remove_child(data: &mut ScriptData, parent: Self, child: Self) {
        // Remove from children.
        let parent_read = parent.0.read().unwrap();
        parent_read
            .children
            .iter()
            .position(|n| n.0.read().unwrap().id == parent_read.id)
            .map(|index| parent.0.write().unwrap().children.remove(index));

        // Remove parent.
        child.0.write().unwrap().parent = None;

        // Update ECS.
        data.commands.push(move |world: &mut World| {
            let child_ent = *child.0.read().unwrap().entity.get().unwrap();
            world.entity_mut(child_ent).remove_parent();
        });
    }
}

fn calc_global_transform(transform: BTransform, node: &NodeRes) -> BTransform {
    let data = node.0.read().unwrap();
    let new_transform = data.transform * transform;

    if let Some(parent) = &data.parent {
        calc_global_transform(new_transform, parent)
    } else {
        new_transform
    }
}
