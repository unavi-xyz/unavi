//! `wired:agent`.

use serde_vrm::vrm1::vrmc_vrm::humanoid::HumanBoneName;
use wasmtime::component::Resource;

use super::{
    HostCtx,
    convert,
    generated::wired::{
        agent::local::{
            Attachment,
            Host,
            HumanoidBone,
            Screen,
        },
        core::math::Transform,
    },
};
use crate::{
    error::ScriptError,
    host::{
        agent,
        scene::DocumentRes,
    },
};

impl Host for HostCtx {
    fn camera_transform(&mut self) -> Result<Transform, ScriptError> {
        agent::camera_transform(&self.host).map(convert::wit_transform)
    }

    fn bone_transform(&mut self, bone: HumanoidBone) -> Result<Option<Transform>, ScriptError> {
        Ok(agent::bone_transform(&self.host, bone_name(bone))?.map(convert::wit_transform))
    }

    fn viewport(&mut self) -> Result<Option<Screen>, ScriptError> {
        Ok(agent::viewport(&self.host)?.map(|screen| Screen {
            width:  screen.width,
            height: screen.height,
            fov:    screen.fov,
        }))
    }

    async fn attach(
        &mut self,
        doc: Resource<DocumentRes>,
        to: Attachment,
        offset: Transform,
    ) -> Result<(), ScriptError> {
        let to = match to {
            Attachment::Camera => agent::Attachment::Camera,
            Attachment::Body => agent::Attachment::Body,
            Attachment::Bone(bone) => agent::Attachment::Bone(bone_name(bone)),
        };
        agent::attach(&self.host, doc.rep(), to, convert::xform(offset)).await
    }
}

/// The two lists are the same VRM set, matched by name except for the thumb,
/// which VRM 1.0 numbers one joint down: its metacarpal is 0.0's proximal.
const fn bone_name(bone: HumanoidBone) -> HumanBoneName {
    match bone {
        HumanoidBone::Hips => HumanBoneName::Hips,
        HumanoidBone::Spine => HumanBoneName::Spine,
        HumanoidBone::Chest => HumanBoneName::Chest,
        HumanoidBone::UpperChest => HumanBoneName::UpperChest,
        HumanoidBone::Neck => HumanBoneName::Neck,
        HumanoidBone::Head => HumanBoneName::Head,
        HumanoidBone::LeftEye => HumanBoneName::LeftEye,
        HumanoidBone::RightEye => HumanBoneName::RightEye,
        HumanoidBone::Jaw => HumanBoneName::Jaw,
        HumanoidBone::LeftShoulder => HumanBoneName::LeftShoulder,
        HumanoidBone::LeftUpperArm => HumanBoneName::LeftUpperArm,
        HumanoidBone::LeftLowerArm => HumanBoneName::LeftLowerArm,
        HumanoidBone::LeftHand => HumanBoneName::LeftHand,
        HumanoidBone::RightShoulder => HumanBoneName::RightShoulder,
        HumanoidBone::RightUpperArm => HumanBoneName::RightUpperArm,
        HumanoidBone::RightLowerArm => HumanBoneName::RightLowerArm,
        HumanoidBone::RightHand => HumanBoneName::RightHand,
        HumanoidBone::LeftUpperLeg => HumanBoneName::LeftUpperLeg,
        HumanoidBone::LeftLowerLeg => HumanBoneName::LeftLowerLeg,
        HumanoidBone::LeftFoot => HumanBoneName::LeftFoot,
        HumanoidBone::LeftToes => HumanBoneName::LeftToes,
        HumanoidBone::RightUpperLeg => HumanBoneName::RightUpperLeg,
        HumanoidBone::RightLowerLeg => HumanBoneName::RightLowerLeg,
        HumanoidBone::RightFoot => HumanBoneName::RightFoot,
        HumanoidBone::RightToes => HumanBoneName::RightToes,
        HumanoidBone::LeftThumbProximal => HumanBoneName::LeftThumbMetacarpal,
        HumanoidBone::LeftThumbIntermediate => HumanBoneName::LeftThumbProximal,
        HumanoidBone::LeftThumbDistal => HumanBoneName::LeftThumbDistal,
        HumanoidBone::LeftIndexProximal => HumanBoneName::LeftIndexProximal,
        HumanoidBone::LeftIndexIntermediate => HumanBoneName::LeftIndexIntermediate,
        HumanoidBone::LeftIndexDistal => HumanBoneName::LeftIndexDistal,
        HumanoidBone::LeftMiddleProximal => HumanBoneName::LeftMiddleProximal,
        HumanoidBone::LeftMiddleIntermediate => HumanBoneName::LeftMiddleIntermediate,
        HumanoidBone::LeftMiddleDistal => HumanBoneName::LeftMiddleDistal,
        HumanoidBone::LeftRingProximal => HumanBoneName::LeftRingProximal,
        HumanoidBone::LeftRingIntermediate => HumanBoneName::LeftRingIntermediate,
        HumanoidBone::LeftRingDistal => HumanBoneName::LeftRingDistal,
        HumanoidBone::LeftLittleProximal => HumanBoneName::LeftLittleProximal,
        HumanoidBone::LeftLittleIntermediate => HumanBoneName::LeftLittleIntermediate,
        HumanoidBone::LeftLittleDistal => HumanBoneName::LeftLittleDistal,
        HumanoidBone::RightThumbProximal => HumanBoneName::RightThumbMetacarpal,
        HumanoidBone::RightThumbIntermediate => HumanBoneName::RightThumbProximal,
        HumanoidBone::RightThumbDistal => HumanBoneName::RightThumbDistal,
        HumanoidBone::RightIndexProximal => HumanBoneName::RightIndexProximal,
        HumanoidBone::RightIndexIntermediate => HumanBoneName::RightIndexIntermediate,
        HumanoidBone::RightIndexDistal => HumanBoneName::RightIndexDistal,
        HumanoidBone::RightMiddleProximal => HumanBoneName::RightMiddleProximal,
        HumanoidBone::RightMiddleIntermediate => HumanBoneName::RightMiddleIntermediate,
        HumanoidBone::RightMiddleDistal => HumanBoneName::RightMiddleDistal,
        HumanoidBone::RightRingProximal => HumanBoneName::RightRingProximal,
        HumanoidBone::RightRingIntermediate => HumanBoneName::RightRingIntermediate,
        HumanoidBone::RightRingDistal => HumanBoneName::RightRingDistal,
        HumanoidBone::RightLittleProximal => HumanBoneName::RightLittleProximal,
        HumanoidBone::RightLittleIntermediate => HumanBoneName::RightLittleIntermediate,
        HumanoidBone::RightLittleDistal => HumanBoneName::RightLittleDistal,
    }
}
