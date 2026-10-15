//! `wired:agent`.

use bevy_vrm::BoneName;
use wasmtime::component::Resource;

use super::{
    HostCtx,
    convert,
    generated::wired::{
        agent::local::{
            Attachment,
            Host,
            HumanoidBone,
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

    async fn attach(
        &mut self,
        doc: Resource<DocumentRes>,
        to: Attachment,
        offset: Transform,
    ) -> Result<(), ScriptError> {
        let to = match to {
            Attachment::Camera => agent::Attachment::Camera,
            Attachment::Bone(bone) => agent::Attachment::Bone(bone_name(bone)),
        };
        agent::attach(&self.host, doc.rep(), to, convert::xform(offset)).await
    }
}

/// The two lists are the same VRM set, matched by name.
const fn bone_name(bone: HumanoidBone) -> BoneName {
    match bone {
        HumanoidBone::Hips => BoneName::Hips,
        HumanoidBone::Spine => BoneName::Spine,
        HumanoidBone::Chest => BoneName::Chest,
        HumanoidBone::UpperChest => BoneName::UpperChest,
        HumanoidBone::Neck => BoneName::Neck,
        HumanoidBone::Head => BoneName::Head,
        HumanoidBone::LeftEye => BoneName::LeftEye,
        HumanoidBone::RightEye => BoneName::RightEye,
        HumanoidBone::Jaw => BoneName::Jaw,
        HumanoidBone::LeftShoulder => BoneName::LeftShoulder,
        HumanoidBone::LeftUpperArm => BoneName::LeftUpperArm,
        HumanoidBone::LeftLowerArm => BoneName::LeftLowerArm,
        HumanoidBone::LeftHand => BoneName::LeftHand,
        HumanoidBone::RightShoulder => BoneName::RightShoulder,
        HumanoidBone::RightUpperArm => BoneName::RightUpperArm,
        HumanoidBone::RightLowerArm => BoneName::RightLowerArm,
        HumanoidBone::RightHand => BoneName::RightHand,
        HumanoidBone::LeftUpperLeg => BoneName::LeftUpperLeg,
        HumanoidBone::LeftLowerLeg => BoneName::LeftLowerLeg,
        HumanoidBone::LeftFoot => BoneName::LeftFoot,
        HumanoidBone::LeftToes => BoneName::LeftToes,
        HumanoidBone::RightUpperLeg => BoneName::RightUpperLeg,
        HumanoidBone::RightLowerLeg => BoneName::RightLowerLeg,
        HumanoidBone::RightFoot => BoneName::RightFoot,
        HumanoidBone::RightToes => BoneName::RightToes,
        HumanoidBone::LeftThumbProximal => BoneName::LeftThumbProximal,
        HumanoidBone::LeftThumbIntermediate => BoneName::LeftThumbIntermediate,
        HumanoidBone::LeftThumbDistal => BoneName::LeftThumbDistal,
        HumanoidBone::LeftIndexProximal => BoneName::LeftIndexProximal,
        HumanoidBone::LeftIndexIntermediate => BoneName::LeftIndexIntermediate,
        HumanoidBone::LeftIndexDistal => BoneName::LeftIndexDistal,
        HumanoidBone::LeftMiddleProximal => BoneName::LeftMiddleProximal,
        HumanoidBone::LeftMiddleIntermediate => BoneName::LeftMiddleIntermediate,
        HumanoidBone::LeftMiddleDistal => BoneName::LeftMiddleDistal,
        HumanoidBone::LeftRingProximal => BoneName::LeftRingProximal,
        HumanoidBone::LeftRingIntermediate => BoneName::LeftRingIntermediate,
        HumanoidBone::LeftRingDistal => BoneName::LeftRingDistal,
        HumanoidBone::LeftLittleProximal => BoneName::LeftLittleProximal,
        HumanoidBone::LeftLittleIntermediate => BoneName::LeftLittleIntermediate,
        HumanoidBone::LeftLittleDistal => BoneName::LeftLittleDistal,
        HumanoidBone::RightThumbProximal => BoneName::RightThumbProximal,
        HumanoidBone::RightThumbIntermediate => BoneName::RightThumbIntermediate,
        HumanoidBone::RightThumbDistal => BoneName::RightThumbDistal,
        HumanoidBone::RightIndexProximal => BoneName::RightIndexProximal,
        HumanoidBone::RightIndexIntermediate => BoneName::RightIndexIntermediate,
        HumanoidBone::RightIndexDistal => BoneName::RightIndexDistal,
        HumanoidBone::RightMiddleProximal => BoneName::RightMiddleProximal,
        HumanoidBone::RightMiddleIntermediate => BoneName::RightMiddleIntermediate,
        HumanoidBone::RightMiddleDistal => BoneName::RightMiddleDistal,
        HumanoidBone::RightRingProximal => BoneName::RightRingProximal,
        HumanoidBone::RightRingIntermediate => BoneName::RightRingIntermediate,
        HumanoidBone::RightRingDistal => BoneName::RightRingDistal,
        HumanoidBone::RightLittleProximal => BoneName::RightLittleProximal,
        HumanoidBone::RightLittleIntermediate => BoneName::RightLittleIntermediate,
        HumanoidBone::RightLittleDistal => BoneName::RightLittleDistal,
    }
}
