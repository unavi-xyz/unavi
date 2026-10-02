//! `wired:agent/local`.

use std::rc::Rc;

use bevy_vrm::BoneName;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

use super::{
    Runtime,
    convert,
    scene::DocumentHandle,
};
use crate::{
    error::ScriptError,
    host::agent,
};

/// The two lists are the same VRM set, matched by name, mirroring native.
fn bone_name(value: &str) -> Result<BoneName, ScriptError> {
    Ok(match value {
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
