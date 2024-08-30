use std::cell::Cell;

use wasm_bridge::component::Resource;
use wasm_bridge_wasi::{ResourceTable, ResourceTableError};

use crate::{
    api::{
        utils::{RefCount, RefCountCell, RefResource},
        wired_scene::{gltf::node::NodeRes, wired::scene::node::HostNode},
    },
    state::StoreState,
};

use super::wired::player::api::{HostPlayer, Skeleton};

pub struct Player {
    ref_count: RefCountCell,
    pub root: Resource<NodeRes>,
    pub skeleton: Skeleton,
}

impl RefCount for Player {
    fn ref_count(&self) -> &Cell<usize> {
        &self.ref_count
    }
}

impl RefResource for Player {}

impl Player {
    pub fn new(data: &mut StoreState) -> anyhow::Result<Resource<Self>> {
        let root = data.new()?;

        let hips = data.new()?;
        data.add_child(
            NodeRes::from_res(&root, &data.table)?,
            NodeRes::from_res(&hips, &data.table)?,
        )?;

        let spine = data.new()?;
        data.add_child(
            NodeRes::from_res(&hips, &data.table)?,
            NodeRes::from_res(&spine, &data.table)?,
        )?;

        let chest = data.new()?;
        data.add_child(
            NodeRes::from_res(&spine, &data.table)?,
            NodeRes::from_res(&chest, &data.table)?,
        )?;

        let upper_chest = data.new()?;
        data.add_child(
            NodeRes::from_res(&chest, &data.table)?,
            NodeRes::from_res(&upper_chest, &data.table)?,
        )?;

        let neck = data.new()?;
        data.add_child(
            NodeRes::from_res(&upper_chest, &data.table)?,
            NodeRes::from_res(&neck, &data.table)?,
        )?;

        let head = data.new()?;
        data.add_child(
            NodeRes::from_res(&neck, &data.table)?,
            NodeRes::from_res(&head, &data.table)?,
        )?;

        let left_shoulder = data.new()?;
        data.add_child(
            NodeRes::from_res(&upper_chest, &data.table)?,
            NodeRes::from_res(&left_shoulder, &data.table)?,
        )?;

        let left_upper_arm = data.new()?;
        data.add_child(
            NodeRes::from_res(&left_shoulder, &data.table)?,
            NodeRes::from_res(&left_upper_arm, &data.table)?,
        )?;

        let left_lower_arm = data.new()?;
        data.add_child(
            NodeRes::from_res(&left_upper_arm, &data.table)?,
            NodeRes::from_res(&left_lower_arm, &data.table)?,
        )?;

        let left_hand = data.new()?;
        data.add_child(
            NodeRes::from_res(&left_lower_arm, &data.table)?,
            NodeRes::from_res(&left_hand, &data.table)?,
        )?;

        let left_upper_leg = data.new()?;
        data.add_child(
            NodeRes::from_res(&hips, &data.table)?,
            NodeRes::from_res(&left_upper_leg, &data.table)?,
        )?;

        let left_lower_leg = data.new()?;
        data.add_child(
            NodeRes::from_res(&left_upper_leg, &data.table)?,
            NodeRes::from_res(&left_lower_leg, &data.table)?,
        )?;

        let left_foot = data.new()?;
        data.add_child(
            NodeRes::from_res(&left_lower_leg, &data.table)?,
            NodeRes::from_res(&left_foot, &data.table)?,
        )?;

        let right_shoulder = data.new()?;
        data.add_child(
            NodeRes::from_res(&upper_chest, &data.table)?,
            NodeRes::from_res(&right_shoulder, &data.table)?,
        )?;

        let right_upper_arm = data.new()?;
        data.add_child(
            NodeRes::from_res(&right_shoulder, &data.table)?,
            NodeRes::from_res(&right_upper_arm, &data.table)?,
        )?;

        let right_lower_arm = data.new()?;
        data.add_child(
            NodeRes::from_res(&right_upper_arm, &data.table)?,
            NodeRes::from_res(&right_lower_arm, &data.table)?,
        )?;

        let right_hand = data.new()?;
        data.add_child(
            NodeRes::from_res(&right_lower_arm, &data.table)?,
            NodeRes::from_res(&right_hand, &data.table)?,
        )?;

        let right_upper_leg = data.new()?;
        data.add_child(
            NodeRes::from_res(&hips, &data.table)?,
            NodeRes::from_res(&right_upper_leg, &data.table)?,
        )?;

        let right_lower_leg = data.new()?;
        data.add_child(
            NodeRes::from_res(&right_upper_leg, &data.table)?,
            NodeRes::from_res(&right_lower_leg, &data.table)?,
        )?;

        let right_foot = data.new()?;
        data.add_child(
            NodeRes::from_res(&right_lower_leg, &data.table)?,
            NodeRes::from_res(&right_foot, &data.table)?,
        )?;

        let player = Self {
            ref_count: RefCountCell::default(),
            root,
            skeleton: Skeleton {
                hips,
                spine,
                chest,
                upper_chest,
                neck,
                head,
                left_shoulder,
                left_upper_arm,
                left_lower_arm,
                left_foot,
                left_hand,
                left_lower_leg,
                left_upper_leg,
                right_shoulder,
                right_upper_arm,
                right_lower_arm,
                right_foot,
                right_hand,
                right_lower_leg,
                right_upper_leg,
            },
        };

        let res = data.table.push(player)?;
        let player = Player::from_res(&res, &data.table)?;

        Ok(player)
    }
}

impl Skeleton {
    /// Clones the [Skeleton] struct, while referencing the same underlying resources.
    pub fn clone_ref(&self, table: &ResourceTable) -> Result<Self, ResourceTableError> {
        Ok(Skeleton {
            hips: NodeRes::from_res(&self.hips, table)?,
            chest: NodeRes::from_res(&self.chest, table)?,
            upper_chest: NodeRes::from_res(&self.upper_chest, table)?,
            neck: NodeRes::from_res(&self.neck, table)?,
            head: NodeRes::from_res(&self.head, table)?,
            left_foot: NodeRes::from_res(&self.left_foot, table)?,
            left_hand: NodeRes::from_res(&self.left_hand, table)?,
            left_lower_arm: NodeRes::from_res(&self.left_lower_arm, table)?,
            left_lower_leg: NodeRes::from_res(&self.left_lower_leg, table)?,
            left_shoulder: NodeRes::from_res(&self.left_shoulder, table)?,
            left_upper_arm: NodeRes::from_res(&self.left_upper_arm, table)?,
            left_upper_leg: NodeRes::from_res(&self.left_upper_leg, table)?,
            right_foot: NodeRes::from_res(&self.right_foot, table)?,
            right_hand: NodeRes::from_res(&self.right_hand, table)?,
            right_lower_arm: NodeRes::from_res(&self.right_lower_arm, table)?,
            right_lower_leg: NodeRes::from_res(&self.right_lower_leg, table)?,
            right_shoulder: NodeRes::from_res(&self.right_shoulder, table)?,
            right_upper_arm: NodeRes::from_res(&self.right_upper_arm, table)?,
            right_upper_leg: NodeRes::from_res(&self.right_upper_leg, table)?,
            spine: NodeRes::from_res(&self.spine, table)?,
        })
    }
}

impl HostPlayer for StoreState {
    fn root(&mut self, self_: Resource<Player>) -> wasm_bridge::Result<Resource<NodeRes>> {
        let player = self.table.get(&self_)?;
        let root = NodeRes::from_res(&player.root, &self.table)?;
        Ok(root)
    }

    fn skeleton(&mut self, self_: Resource<Player>) -> wasm_bridge::Result<Skeleton> {
        let player = self.table.get(&self_)?;
        let skeleton = player.skeleton.clone_ref(&self.table)?;
        Ok(skeleton)
    }

    fn drop(&mut self, _rep: Resource<Player>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}
