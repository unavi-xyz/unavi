use std::cell::Cell;

use wasm_bridge::component::Resource;
use wasm_bridge_wasi::{ResourceTable, ResourceTableError};

use crate::{
    api::{
        utils::{RefCount, RefCountCell, RefResource},
        wired_scene::gltf::node::NodeRes,
    },
    state::StoreState,
};

use super::wired::player::api::{HostPlayer, Skeleton};

pub struct Player {
    ref_count: RefCountCell,
    skeleton: Skeleton,
}

impl RefCount for Player {
    fn ref_count(&self) -> &Cell<usize> {
        &self.ref_count
    }
}

impl RefResource for Player {}

impl Player {
    pub fn new(table: &mut ResourceTable) -> Result<Resource<Self>, ResourceTableError> {
        let player = Self {
            ref_count: RefCountCell::default(),
            skeleton: Skeleton {
                head: Resource::new_own(0),
                hips: Resource::new_own(0),
                left_foot: Resource::new_own(0),
                left_hand: Resource::new_own(0),
                left_lower_arm: Resource::new_own(0),
                left_lower_leg: Resource::new_own(0),
                left_upper_arm: Resource::new_own(0),
                left_upper_leg: Resource::new_own(0),
                right_foot: Resource::new_own(0),
                right_hand: Resource::new_own(0),
                right_lower_arm: Resource::new_own(0),
                right_lower_leg: Resource::new_own(0),
                right_upper_arm: Resource::new_own(0),
                right_upper_leg: Resource::new_own(0),
                spine: Resource::new_own(0),
            },
        };

        let res = table.push(player)?;

        let skeleton = Skeleton {
            head: table.push_child(NodeRes::default(), &res)?,
            hips: table.push_child(NodeRes::default(), &res)?,
            left_foot: table.push_child(NodeRes::default(), &res)?,
            left_hand: table.push_child(NodeRes::default(), &res)?,
            left_lower_arm: table.push_child(NodeRes::default(), &res)?,
            left_lower_leg: table.push_child(NodeRes::default(), &res)?,
            left_upper_arm: table.push_child(NodeRes::default(), &res)?,
            left_upper_leg: table.push_child(NodeRes::default(), &res)?,
            right_foot: table.push_child(NodeRes::default(), &res)?,
            right_hand: table.push_child(NodeRes::default(), &res)?,
            right_lower_arm: table.push_child(NodeRes::default(), &res)?,
            right_lower_leg: table.push_child(NodeRes::default(), &res)?,
            right_upper_arm: table.push_child(NodeRes::default(), &res)?,
            right_upper_leg: table.push_child(NodeRes::default(), &res)?,
            spine: table.push_child(NodeRes::default(), &res)?,
        };

        let res_mut = table.get_mut(&res)?;
        res_mut.skeleton = skeleton;

        Player::from_res(&res, table)
    }
}

impl Skeleton {
    fn clone(&self, table: &ResourceTable) -> Result<Self, ResourceTableError> {
        Ok(Skeleton {
            head: NodeRes::from_res(&self.head, table)?,
            hips: NodeRes::from_res(&self.hips, table)?,
            left_foot: NodeRes::from_res(&self.left_foot, table)?,
            left_hand: NodeRes::from_res(&self.left_hand, table)?,
            left_lower_arm: NodeRes::from_res(&self.left_lower_arm, table)?,
            left_lower_leg: NodeRes::from_res(&self.left_lower_leg, table)?,
            left_upper_arm: NodeRes::from_res(&self.left_upper_arm, table)?,
            left_upper_leg: NodeRes::from_res(&self.left_upper_leg, table)?,
            right_foot: NodeRes::from_res(&self.right_foot, table)?,
            right_hand: NodeRes::from_res(&self.right_hand, table)?,
            right_lower_arm: NodeRes::from_res(&self.right_lower_arm, table)?,
            right_lower_leg: NodeRes::from_res(&self.right_lower_leg, table)?,
            right_upper_arm: NodeRes::from_res(&self.right_upper_arm, table)?,
            right_upper_leg: NodeRes::from_res(&self.right_upper_leg, table)?,
            spine: NodeRes::from_res(&self.spine, table)?,
        })
    }
}

impl HostPlayer for StoreState {
    fn skeleton(&mut self, self_: Resource<Player>) -> wasm_bridge::Result<Skeleton> {
        let player = self.table.get(&self_)?;
        let skeleton = player.skeleton.clone(&self.table)?;
        Ok(skeleton)
    }

    fn drop(&mut self, _rep: Resource<Player>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}
