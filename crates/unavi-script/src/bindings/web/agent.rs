//! `wired:agent/local`.

use std::rc::Rc;

use serde_vrm::vrm1::vrmc_vrm::humanoid::HumanBoneName;
use wasm_bindgen::prelude::*;

use super::{
    Runtime,
    convert,
    inline_promise as future_to_promise,
    scene::DocumentHandle,
};
use crate::{
    error::ScriptError,
    host::agent,
};

/// The two lists are the same VRM set, matched by name except for the thumb,
/// which VRM 1.0 numbers one joint down, mirroring native.
fn bone_name(value: &str) -> Result<HumanBoneName, ScriptError> {
    Ok(match value {
        "hips" => HumanBoneName::Hips,
        "spine" => HumanBoneName::Spine,
        "chest" => HumanBoneName::Chest,
        "upper-chest" => HumanBoneName::UpperChest,
        "neck" => HumanBoneName::Neck,
        "head" => HumanBoneName::Head,
        "left-eye" => HumanBoneName::LeftEye,
        "right-eye" => HumanBoneName::RightEye,
        "jaw" => HumanBoneName::Jaw,
        "left-shoulder" => HumanBoneName::LeftShoulder,
        "left-upper-arm" => HumanBoneName::LeftUpperArm,
        "left-lower-arm" => HumanBoneName::LeftLowerArm,
        "left-hand" => HumanBoneName::LeftHand,
        "right-shoulder" => HumanBoneName::RightShoulder,
        "right-upper-arm" => HumanBoneName::RightUpperArm,
        "right-lower-arm" => HumanBoneName::RightLowerArm,
        "right-hand" => HumanBoneName::RightHand,
        "left-upper-leg" => HumanBoneName::LeftUpperLeg,
        "left-lower-leg" => HumanBoneName::LeftLowerLeg,
        "left-foot" => HumanBoneName::LeftFoot,
        "left-toes" => HumanBoneName::LeftToes,
        "right-upper-leg" => HumanBoneName::RightUpperLeg,
        "right-lower-leg" => HumanBoneName::RightLowerLeg,
        "right-foot" => HumanBoneName::RightFoot,
        "right-toes" => HumanBoneName::RightToes,
        "left-thumb-proximal" => HumanBoneName::LeftThumbMetacarpal,
        "left-thumb-intermediate" => HumanBoneName::LeftThumbProximal,
        "left-thumb-distal" => HumanBoneName::LeftThumbDistal,
        "left-index-proximal" => HumanBoneName::LeftIndexProximal,
        "left-index-intermediate" => HumanBoneName::LeftIndexIntermediate,
        "left-index-distal" => HumanBoneName::LeftIndexDistal,
        "left-middle-proximal" => HumanBoneName::LeftMiddleProximal,
        "left-middle-intermediate" => HumanBoneName::LeftMiddleIntermediate,
        "left-middle-distal" => HumanBoneName::LeftMiddleDistal,
        "left-ring-proximal" => HumanBoneName::LeftRingProximal,
        "left-ring-intermediate" => HumanBoneName::LeftRingIntermediate,
        "left-ring-distal" => HumanBoneName::LeftRingDistal,
        "left-little-proximal" => HumanBoneName::LeftLittleProximal,
        "left-little-intermediate" => HumanBoneName::LeftLittleIntermediate,
        "left-little-distal" => HumanBoneName::LeftLittleDistal,
        "right-thumb-proximal" => HumanBoneName::RightThumbMetacarpal,
        "right-thumb-intermediate" => HumanBoneName::RightThumbProximal,
        "right-thumb-distal" => HumanBoneName::RightThumbDistal,
        "right-index-proximal" => HumanBoneName::RightIndexProximal,
        "right-index-intermediate" => HumanBoneName::RightIndexIntermediate,
        "right-index-distal" => HumanBoneName::RightIndexDistal,
        "right-middle-proximal" => HumanBoneName::RightMiddleProximal,
        "right-middle-intermediate" => HumanBoneName::RightMiddleIntermediate,
        "right-middle-distal" => HumanBoneName::RightMiddleDistal,
        "right-ring-proximal" => HumanBoneName::RightRingProximal,
        "right-ring-intermediate" => HumanBoneName::RightRingIntermediate,
        "right-ring-distal" => HumanBoneName::RightRingDistal,
        "right-little-proximal" => HumanBoneName::RightLittleProximal,
        "right-little-intermediate" => HumanBoneName::RightLittleIntermediate,
        "right-little-distal" => HumanBoneName::RightLittleDistal,
        _ => return Err(ScriptError::invalid("not a humanoid bone")),
    })
}

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(js_name = "cameraTransform")]
    pub fn camera_transform(&self) -> Result<JsValue, JsValue> {
        agent::camera_transform(&self.host.borrow())
            .map(convert::wit_transform)
            .map_err(convert::raise)
    }

    #[wasm_bindgen(js_name = "boneTransform")]
    pub fn bone_transform(&self, bone: &str) -> Result<JsValue, JsValue> {
        let bone = bone_name(bone).map_err(convert::raise)?;
        agent::bone_transform(&self.host.borrow(), bone)
            .map(|t| t.map_or(JsValue::UNDEFINED, convert::wit_transform))
            .map_err(convert::raise)
    }

    pub fn attach(
        &self,
        document: &DocumentHandle,
        to: JsValue,
        offset: JsValue,
    ) -> js_sys::Promise {
        let host = Rc::clone(&self.host);
        let doc = document.rep();
        let parsed = (|| -> Result<_, ScriptError> {
            let to = match convert::tag(&to).as_str() {
                "camera" => agent::Attachment::Camera,
                "body" => agent::Attachment::Body,
                "bone" => {
                    let bone = convert::val(&to)
                        .as_string()
                        .ok_or_else(|| ScriptError::invalid("a bone is a string"))?;
                    agent::Attachment::Bone(bone_name(&bone)?)
                }
                _ => return Err(ScriptError::invalid("not an attachment")),
            };
            Ok((to, convert::xform(offset)?))
        })();
        let (to, offset) = match parsed {
            Ok(v) => v,
            Err(err) => return js_sys::Promise::reject(&convert::raise(err)),
        };
        future_to_promise(async move {
            let host = host.borrow();
            agent::attach(&host, doc, to, offset)
                .await
                .map(|()| JsValue::UNDEFINED)
                .map_err(convert::raise)
        })
    }
}
