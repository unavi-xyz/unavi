//! Mixamo's bone-naming convention, used to retarget downloaded Mixamo clips
//! onto a VRM's humanoid bones.

use std::sync::LazyLock;

use bevy::platform::collections::HashMap;
use bevy_vrm::BoneName;

/// `(VRM bone, Mixamo node name)` pairs for every bone a Mixamo rig covers.
/// Eyes and jaw are Mixamo has no equivalent for and are tracked separately.
const MIXAMO_BONES: &[(BoneName, &str)] = &[
    (BoneName::Hips, "mixamorig:Hips"),
    (BoneName::LeftUpperLeg, "mixamorig:LeftUpLeg"),
    (BoneName::LeftLowerLeg, "mixamorig:LeftLeg"),
    (BoneName::LeftFoot, "mixamorig:LeftFoot"),
    (BoneName::LeftToes, "mixamorig:LeftToeBase"),
    (BoneName::RightUpperLeg, "mixamorig:RightUpLeg"),
    (BoneName::RightLowerLeg, "mixamorig:RightLeg"),
    (BoneName::RightFoot, "mixamorig:RightFoot"),
    (BoneName::RightToes, "mixamorig:RightToeBase"),
    (BoneName::Spine, "mixamorig:Spine"),
    (BoneName::Chest, "mixamorig:Spine1"),
    (BoneName::UpperChest, "mixamorig:Spine2"),
    (BoneName::LeftShoulder, "mixamorig:LeftShoulder"),
    (BoneName::LeftUpperArm, "mixamorig:LeftArm"),
    (BoneName::LeftLowerArm, "mixamorig:LeftForeArm"),
    (BoneName::LeftHand, "mixamorig:LeftHand"),
    (BoneName::LeftThumbProximal, "mixamorig:LeftHandThumb1"),
    (BoneName::LeftThumbIntermediate, "mixamorig:LeftHandThumb2"),
    (BoneName::LeftThumbDistal, "mixamorig:LeftHandThumb3"),
    (BoneName::LeftIndexProximal, "mixamorig:LeftHandIndex1"),
    (BoneName::LeftIndexIntermediate, "mixamorig:LeftHandIndex2"),
    (BoneName::LeftIndexDistal, "mixamorig:LeftHandIndex3"),
    (BoneName::LeftMiddleProximal, "mixamorig:LeftHandMiddle1"),
    (
        BoneName::LeftMiddleIntermediate,
        "mixamorig:LeftHandMiddle2",
    ),
    (BoneName::LeftMiddleDistal, "mixamorig:LeftHandMiddle3"),
    (BoneName::LeftRingProximal, "mixamorig:LeftHandRing1"),
    (BoneName::LeftRingIntermediate, "mixamorig:LeftHandRing2"),
    (BoneName::LeftRingDistal, "mixamorig:LeftHandRing3"),
    (BoneName::LeftLittleProximal, "mixamorig:LeftHandPinky1"),
    (BoneName::LeftLittleIntermediate, "mixamorig:LeftHandPinky2"),
    (BoneName::LeftLittleDistal, "mixamorig:LeftHandPinky3"),
    (BoneName::RightShoulder, "mixamorig:RightShoulder"),
    (BoneName::RightUpperArm, "mixamorig:RightArm"),
    (BoneName::RightLowerArm, "mixamorig:RightForeArm"),
    (BoneName::RightHand, "mixamorig:RightHand"),
    (BoneName::RightThumbProximal, "mixamorig:RightHandThumb1"),
    (
        BoneName::RightThumbIntermediate,
        "mixamorig:RightHandThumb2",
    ),
    (BoneName::RightThumbDistal, "mixamorig:RightHandThumb3"),
    (BoneName::RightIndexProximal, "mixamorig:RightHandIndex1"),
    (
        BoneName::RightIndexIntermediate,
        "mixamorig:RightHandIndex2",
    ),
    (BoneName::RightIndexDistal, "mixamorig:RightHandIndex3"),
    (BoneName::RightMiddleProximal, "mixamorig:RightHandMiddle1"),
    (
        BoneName::RightMiddleIntermediate,
        "mixamorig:RightHandMiddle2",
    ),
    (BoneName::RightMiddleDistal, "mixamorig:RightHandMiddle3"),
    (BoneName::RightRingProximal, "mixamorig:RightHandRing1"),
    (BoneName::RightRingIntermediate, "mixamorig:RightHandRing2"),
    (BoneName::RightRingDistal, "mixamorig:RightHandRing3"),
    (BoneName::RightLittleProximal, "mixamorig:RightHandPinky1"),
    (
        BoneName::RightLittleIntermediate,
        "mixamorig:RightHandPinky2",
    ),
    (BoneName::RightLittleDistal, "mixamorig:RightHandPinky3"),
    (BoneName::Neck, "mixamorig:Neck"),
    (BoneName::Head, "mixamorig:Head"),
];

/// Mixamo node name to VRM bone, for retargeting a clip's channels in one
/// lookup per channel instead of a linear scan of [`MIXAMO_BONES`].
pub static MIXAMO_BONE_BY_NAME: LazyLock<HashMap<&'static str, BoneName>> = LazyLock::new(|| {
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
