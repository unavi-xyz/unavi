//! Mixamo's bone-naming convention, used to retarget downloaded Mixamo clips
//! onto a VRM's humanoid bones.

use std::sync::LazyLock;

use bevy::platform::collections::HashMap;
use serde_vrm::vrm1::vrmc_vrm::humanoid::HumanBoneName;

/// `(VRM bone, Mixamo node name)` pairs for every bone a Mixamo rig covers.
/// Eyes and jaw are Mixamo has no equivalent for and are tracked separately.
const MIXAMO_BONES: &[(HumanBoneName, &str)] = &[
    (HumanBoneName::Hips, "mixamorig:Hips"),
    (HumanBoneName::LeftUpperLeg, "mixamorig:LeftUpLeg"),
    (HumanBoneName::LeftLowerLeg, "mixamorig:LeftLeg"),
    (HumanBoneName::LeftFoot, "mixamorig:LeftFoot"),
    (HumanBoneName::LeftToes, "mixamorig:LeftToeBase"),
    (HumanBoneName::RightUpperLeg, "mixamorig:RightUpLeg"),
    (HumanBoneName::RightLowerLeg, "mixamorig:RightLeg"),
    (HumanBoneName::RightFoot, "mixamorig:RightFoot"),
    (HumanBoneName::RightToes, "mixamorig:RightToeBase"),
    (HumanBoneName::Spine, "mixamorig:Spine"),
    (HumanBoneName::Chest, "mixamorig:Spine1"),
    (HumanBoneName::UpperChest, "mixamorig:Spine2"),
    (HumanBoneName::LeftShoulder, "mixamorig:LeftShoulder"),
    (HumanBoneName::LeftUpperArm, "mixamorig:LeftArm"),
    (HumanBoneName::LeftLowerArm, "mixamorig:LeftForeArm"),
    (HumanBoneName::LeftHand, "mixamorig:LeftHand"),
    // VRM 1.0 numbers the thumb one joint down: its metacarpal is Mixamo's
    // first thumb joint.
    (
        HumanBoneName::LeftThumbMetacarpal,
        "mixamorig:LeftHandThumb1",
    ),
    (HumanBoneName::LeftThumbProximal, "mixamorig:LeftHandThumb2"),
    (HumanBoneName::LeftThumbDistal, "mixamorig:LeftHandThumb3"),
    (HumanBoneName::LeftIndexProximal, "mixamorig:LeftHandIndex1"),
    (
        HumanBoneName::LeftIndexIntermediate,
        "mixamorig:LeftHandIndex2",
    ),
    (HumanBoneName::LeftIndexDistal, "mixamorig:LeftHandIndex3"),
    (
        HumanBoneName::LeftMiddleProximal,
        "mixamorig:LeftHandMiddle1",
    ),
    (
        HumanBoneName::LeftMiddleIntermediate,
        "mixamorig:LeftHandMiddle2",
    ),
    (HumanBoneName::LeftMiddleDistal, "mixamorig:LeftHandMiddle3"),
    (HumanBoneName::LeftRingProximal, "mixamorig:LeftHandRing1"),
    (
        HumanBoneName::LeftRingIntermediate,
        "mixamorig:LeftHandRing2",
    ),
    (HumanBoneName::LeftRingDistal, "mixamorig:LeftHandRing3"),
    (
        HumanBoneName::LeftLittleProximal,
        "mixamorig:LeftHandPinky1",
    ),
    (
        HumanBoneName::LeftLittleIntermediate,
        "mixamorig:LeftHandPinky2",
    ),
    (HumanBoneName::LeftLittleDistal, "mixamorig:LeftHandPinky3"),
    (HumanBoneName::RightShoulder, "mixamorig:RightShoulder"),
    (HumanBoneName::RightUpperArm, "mixamorig:RightArm"),
    (HumanBoneName::RightLowerArm, "mixamorig:RightForeArm"),
    (HumanBoneName::RightHand, "mixamorig:RightHand"),
    (
        HumanBoneName::RightThumbMetacarpal,
        "mixamorig:RightHandThumb1",
    ),
    (
        HumanBoneName::RightThumbProximal,
        "mixamorig:RightHandThumb2",
    ),
    (HumanBoneName::RightThumbDistal, "mixamorig:RightHandThumb3"),
    (
        HumanBoneName::RightIndexProximal,
        "mixamorig:RightHandIndex1",
    ),
    (
        HumanBoneName::RightIndexIntermediate,
        "mixamorig:RightHandIndex2",
    ),
    (HumanBoneName::RightIndexDistal, "mixamorig:RightHandIndex3"),
    (
        HumanBoneName::RightMiddleProximal,
        "mixamorig:RightHandMiddle1",
    ),
    (
        HumanBoneName::RightMiddleIntermediate,
        "mixamorig:RightHandMiddle2",
    ),
    (
        HumanBoneName::RightMiddleDistal,
        "mixamorig:RightHandMiddle3",
    ),
    (HumanBoneName::RightRingProximal, "mixamorig:RightHandRing1"),
    (
        HumanBoneName::RightRingIntermediate,
        "mixamorig:RightHandRing2",
    ),
    (HumanBoneName::RightRingDistal, "mixamorig:RightHandRing3"),
    (
        HumanBoneName::RightLittleProximal,
        "mixamorig:RightHandPinky1",
    ),
    (
        HumanBoneName::RightLittleIntermediate,
        "mixamorig:RightHandPinky2",
    ),
    (
        HumanBoneName::RightLittleDistal,
        "mixamorig:RightHandPinky3",
    ),
    (HumanBoneName::Neck, "mixamorig:Neck"),
    (HumanBoneName::Head, "mixamorig:Head"),
];

/// Mixamo node name to VRM bone, for retargeting a clip's channels in one
/// lookup per channel instead of a linear scan of [`MIXAMO_BONES`].
pub static MIXAMO_BONE_BY_NAME: LazyLock<HashMap<&'static str, HumanBoneName>> =
    LazyLock::new(|| {
        MIXAMO_BONES
            .iter()
            .map(|(bone, name)| (*name, *bone))
            .collect()
    });

#[cfg(test)]
mod tests {
    use super::{
        MIXAMO_BONE_BY_NAME,
        MIXAMO_BONES,
    };

    #[test]
    fn every_table_entry_is_reachable_by_name() {
        for (bone, name) in MIXAMO_BONES {
            assert_eq!(MIXAMO_BONE_BY_NAME.get(name), Some(bone));
        }
    }

    #[test]
    fn mixamo_names_are_unique() {
        assert_eq!(MIXAMO_BONE_BY_NAME.len(), MIXAMO_BONES.len());
    }
}
