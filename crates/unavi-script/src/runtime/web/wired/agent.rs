use std::sync::Arc;

use wasm_bindgen::prelude::*;

use crate::runtime::{
    Runtime,
    shared::{self, Api, wired::agent::BoneName},
};

use super::scene::node::NodeHandle;

#[wasm_bindgen]
pub struct AgentHandle {
    rep: u32,
    api: Arc<Api>,
}

impl AgentHandle {
    pub fn new(rep: u32, api: Arc<Api>) -> Self {
        Self { rep, api }
    }
}

impl Drop for AgentHandle {
    fn drop(&mut self) {
        if self.rep != u32::MAX {
            let _ = shared::wired::agent::on_drop(&self.api, self.rep);
        }
    }
}

fn js_to_bone_name(s: &str) -> Option<BoneName> {
    Some(match s {
        "hips" => BoneName::Hips,
        "spine" => BoneName::Spine,
        "chest" => BoneName::Chest,
        "upper-chest" => BoneName::UpperChest,
        "neck" => BoneName::Neck,
        "head" => BoneName::Head,
        "left-eye" => BoneName::LeftEye,
        "right-eye" => BoneName::RightEye,
        "jaw" => BoneName::Jaw,
        "left-shoulder" => BoneName::LeftShoulder,
        "left-upper-arm" => BoneName::LeftUpperArm,
        "left-lower-arm" => BoneName::LeftLowerArm,
        "left-hand" => BoneName::LeftHand,
        "right-shoulder" => BoneName::RightShoulder,
        "right-upper-arm" => BoneName::RightUpperArm,
        "right-lower-arm" => BoneName::RightLowerArm,
        "right-hand" => BoneName::RightHand,
        "left-upper-leg" => BoneName::LeftUpperLeg,
        "left-lower-leg" => BoneName::LeftLowerLeg,
        "left-foot" => BoneName::LeftFoot,
        "left-toes" => BoneName::LeftToes,
        "right-upper-leg" => BoneName::RightUpperLeg,
        "right-lower-leg" => BoneName::RightLowerLeg,
        "right-foot" => BoneName::RightFoot,
        "right-toes" => BoneName::RightToes,
        "left-thumb-proximal" => BoneName::LeftThumbProximal,
        "left-thumb-intermediate" => BoneName::LeftThumbIntermediate,
        "left-thumb-distal" => BoneName::LeftThumbDistal,
        "left-index-proximal" => BoneName::LeftIndexProximal,
        "left-index-intermediate" => BoneName::LeftIndexIntermediate,
        "left-index-distal" => BoneName::LeftIndexDistal,
        "left-middle-proximal" => BoneName::LeftMiddleProximal,
        "left-middle-intermediate" => BoneName::LeftMiddleIntermediate,
        "left-middle-distal" => BoneName::LeftMiddleDistal,
        "left-ring-proximal" => BoneName::LeftRingProximal,
        "left-ring-intermediate" => BoneName::LeftRingIntermediate,
        "left-ring-distal" => BoneName::LeftRingDistal,
        "left-little-proximal" => BoneName::LeftLittleProximal,
        "left-little-intermediate" => BoneName::LeftLittleIntermediate,
        "left-little-distal" => BoneName::LeftLittleDistal,
        "right-thumb-proximal" => BoneName::RightThumbProximal,
        "right-thumb-intermediate" => BoneName::RightThumbIntermediate,
        "right-thumb-distal" => BoneName::RightThumbDistal,
        "right-index-proximal" => BoneName::RightIndexProximal,
        "right-index-intermediate" => BoneName::RightIndexIntermediate,
        "right-index-distal" => BoneName::RightIndexDistal,
        "right-middle-proximal" => BoneName::RightMiddleProximal,
        "right-middle-intermediate" => BoneName::RightMiddleIntermediate,
        "right-middle-distal" => BoneName::RightMiddleDistal,
        "right-ring-proximal" => BoneName::RightRingProximal,
        "right-ring-intermediate" => BoneName::RightRingIntermediate,
        "right-ring-distal" => BoneName::RightRingDistal,
        "right-little-proximal" => BoneName::RightLittleProximal,
        "right-little-intermediate" => BoneName::RightLittleIntermediate,
        "right-little-distal" => BoneName::RightLittleDistal,
        _ => return None,
    })
}

#[wasm_bindgen]
impl AgentHandle {
    pub fn bone(&self, name: String) -> Option<NodeHandle> {
        let bone = js_to_bone_name(&name)?;
        let rep = shared::wired::agent::bone(&self.api, self.rep, bone)
            .ok()
            .flatten()?;
        Some(NodeHandle::new(rep, Arc::clone(&self.api)))
    }
}

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(js_name = "wiredAgentClass")]
    pub fn wired_agent_class(&self) -> JsValue {
        let handle = AgentHandle::new(u32::MAX, self.api.clone());
        let js = JsValue::from(handle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    #[wasm_bindgen(js_name = "wiredAgentLocalAgent")]
    pub fn wired_agent_local_agent(&self) -> AgentHandle {
        let rep = shared::wired::agent::local_agent(&self.api).unwrap_or(u32::MAX);
        AgentHandle::new(rep, self.api.clone())
    }

    #[wasm_bindgen(js_name = "wiredAgentLocalCamera")]
    pub fn wired_agent_local_camera(&self) -> NodeHandle {
        let rep = shared::wired::agent::local_camera(&self.api).unwrap_or(u32::MAX);
        NodeHandle::new(rep, self.api.clone())
    }
}
