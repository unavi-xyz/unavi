use std::sync::{Arc, RwLock};

use wasm_bridge::component::Resource;
use wasm_bridge_wasi::ResourceTable;

use crate::{
    api::wired::scene::{bindings::node::HostNode, nodes::base::NodeRes},
    data::ScriptData,
};

use super::bindings::api::{HostPlayer, Node, Skeleton};

impl HostPlayer for ScriptData {
    fn root(&mut self, self_: Resource<PlayerRes>) -> wasm_bridge::Result<Resource<Node>> {
        let data = self.table.get(&self_)?.0.read().unwrap().root.clone();
        let res = self.table.push(data)?;
        Ok(res)
    }

    fn skeleton(&mut self, self_: Resource<PlayerRes>) -> wasm_bridge::Result<Skeleton> {
        let data = self.table.get(&self_)?.0.read().unwrap().skeleton.clone();
        data.to_resource(&mut self.table)
    }

    fn drop(&mut self, _rep: Resource<PlayerRes>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct PlayerRes(pub Arc<RwLock<PlayerData>>);

pub struct PlayerData {
    pub root: NodeRes,
    pub skeleton: SkeletonData,
}

#[derive(Clone)]
pub struct SkeletonData {
    hips: NodeRes,
    spine: NodeRes,
    chest: NodeRes,
    upper_chest: NodeRes,
    neck: NodeRes,
    head: NodeRes,
    left_shoulder: NodeRes,
    left_upper_arm: NodeRes,
    left_lower_arm: NodeRes,
    left_foot: NodeRes,
    left_hand: NodeRes,
    left_lower_leg: NodeRes,
    left_upper_leg: NodeRes,
    right_shoulder: NodeRes,
    right_upper_arm: NodeRes,
    right_lower_arm: NodeRes,
    right_foot: NodeRes,
    right_hand: NodeRes,
    right_lower_leg: NodeRes,
    right_upper_leg: NodeRes,
}

impl PlayerRes {
    pub fn new(data: &mut ScriptData) -> Self {
        let root = NodeRes::new(data);
        let hips = NodeRes::new(data);
        NodeRes::add_child(data, root.clone(), hips.clone());

        let spine = NodeRes::new(data);
        NodeRes::add_child(data, hips.clone(), spine.clone());

        let chest = NodeRes::new(data);
        NodeRes::add_child(data, spine.clone(), chest.clone());

        let upper_chest = NodeRes::new(data);
        NodeRes::add_child(data, chest.clone(), upper_chest.clone());

        let neck = NodeRes::new(data);
        NodeRes::add_child(data, upper_chest.clone(), neck.clone());

        let head = NodeRes::new(data);
        NodeRes::add_child(data, neck.clone(), head.clone());

        let left_shoulder = NodeRes::new(data);
        NodeRes::add_child(data, upper_chest.clone(), left_shoulder.clone());

        let left_upper_arm = NodeRes::new(data);
        NodeRes::add_child(data, left_shoulder.clone(), left_upper_arm.clone());

        let left_lower_arm = NodeRes::new(data);
        NodeRes::add_child(data, left_upper_arm.clone(), left_lower_arm.clone());

        let left_hand = NodeRes::new(data);
        NodeRes::add_child(data, left_lower_arm.clone(), left_hand.clone());

        let left_upper_leg = NodeRes::new(data);
        NodeRes::add_child(data, hips.clone(), left_upper_leg.clone());

        let left_lower_leg = NodeRes::new(data);
        NodeRes::add_child(data, left_upper_leg.clone(), left_lower_leg.clone());

        let left_foot = NodeRes::new(data);
        NodeRes::add_child(data, left_lower_leg.clone(), left_foot.clone());

        let right_shoulder = NodeRes::new(data);
        NodeRes::add_child(data, upper_chest.clone(), right_shoulder.clone());

        let right_upper_arm = NodeRes::new(data);
        NodeRes::add_child(data, right_shoulder.clone(), right_upper_arm.clone());

        let right_lower_arm = NodeRes::new(data);
        NodeRes::add_child(data, right_upper_arm.clone(), right_lower_arm.clone());

        let right_hand = NodeRes::new(data);
        NodeRes::add_child(data, right_lower_arm.clone(), right_hand.clone());

        let right_upper_leg = NodeRes::new(data);
        NodeRes::add_child(data, hips.clone(), right_upper_leg.clone());

        let right_lower_leg = NodeRes::new(data);
        NodeRes::add_child(data, right_upper_leg.clone(), right_lower_leg.clone());

        let right_foot = NodeRes::new(data);
        NodeRes::add_child(data, right_lower_leg.clone(), right_foot.clone());

        PlayerRes(Arc::new(RwLock::new(PlayerData {
            root,
            skeleton: SkeletonData {
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
        })))
    }
}

impl SkeletonData {
    pub fn to_resource(self, table: &mut ResourceTable) -> wasm_bridge::Result<Skeleton> {
        Ok(Skeleton {
            hips: table.push(self.hips)?,
            chest: table.push(self.chest)?,
            upper_chest: table.push(self.upper_chest)?,
            neck: table.push(self.neck)?,
            head: table.push(self.head)?,
            left_foot: table.push(self.left_foot)?,
            left_hand: table.push(self.left_hand)?,
            left_lower_arm: table.push(self.left_lower_arm)?,
            left_lower_leg: table.push(self.left_lower_leg)?,
            left_shoulder: table.push(self.left_shoulder)?,
            left_upper_arm: table.push(self.left_upper_arm)?,
            left_upper_leg: table.push(self.left_upper_leg)?,
            right_foot: table.push(self.right_foot)?,
            right_hand: table.push(self.right_hand)?,
            right_lower_arm: table.push(self.right_lower_arm)?,
            right_lower_leg: table.push(self.right_lower_leg)?,
            right_shoulder: table.push(self.right_shoulder)?,
            right_upper_arm: table.push(self.right_upper_arm)?,
            right_upper_leg: table.push(self.right_upper_leg)?,
            spine: table.push(self.spine)?,
        })
    }
}
