use bevy::utils::HashMap;

pub fn mixamo_vrm_bone_names() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();

    map.insert("Hips", "root");
    map.insert("Spine", "spine");
    map.insert("Spine1", "chest");
    map.insert("Spine2", "upperChest");
    map.insert("Neck", "neck");
    map.insert("Head", "head");
    map.insert("LeftShoulder", "leftShoulder");
    map.insert("LeftArm", "leftUpperArm");
    map.insert("LeftForeArm", "leftLowerArm");
    map.insert("LeftHand", "leftHand");
    map.insert("LeftHandThumb1", "leftThumbProximal");
    map.insert("LeftHandThumb2", "leftThumbIntermediate");
    map.insert("LeftHandThumb3", "leftThumbDistal");
    map.insert("LeftHandIndex1", "leftIndexProximal");
    map.insert("LeftHandIndex2", "leftIndexIntermediate");
    map.insert("LeftHandIndex3", "leftIndexDistal");
    map.insert("LeftHandMiddle1", "leftMiddleProximal");
    map.insert("LeftHandMiddle2", "leftMiddleIntermediate");
    map.insert("LeftHandMiddle3", "leftMiddleDistal");
    map.insert("LeftHandRing1", "leftRingProximal");
    map.insert("LeftHandRing2", "leftRingIntermediate");
    map.insert("LeftHandRing3", "leftRingDistal");
    map.insert("LeftHandPinky1", "leftLittleProximal");
    map.insert("LeftHandPinky2", "leftLittleIntermediate");
    map.insert("LeftHandPinky3", "leftLittleDistal");
    map.insert("RightShoulder", "rightShoulder");
    map.insert("RightArm", "rightUpperArm");
    map.insert("RightForeArm", "rightLowerArm");
    map.insert("RightHand", "rightHand");
    map.insert("RightHandThumb1", "rightThumbProximal");
    map.insert("RightHandThumb2", "rightThumbIntermediate");
    map.insert("RightHandThumb3", "rightThumbDistal");
    map.insert("RightHandIndex1", "rightIndexProximal");
    map.insert("RightHandIndex2", "rightIndexIntermediate");
    map.insert("RightHandIndex3", "rightIndexDistal");
    map.insert("RightHandMiddle1", "rightMiddleProximal");
    map.insert("RightHandMiddle2", "rightMiddleIntermediate");
    map.insert("RightHandMiddle3", "rightMiddleDistal");
    map.insert("RightHandRing1", "rightRingProximal");
    map.insert("RightHandRing2", "rightRingIntermediate");
    map.insert("RightHandRing3", "rightRingDistal");
    map.insert("RightHandPinky1", "rightLittleProximal");
    map.insert("RightHandPinky2", "rightLittleIntermediate");
    map.insert("RightHandPinky3", "rightLittleDistal");
    map.insert("LeftUpLeg", "leftUpperLeg");
    map.insert("LeftLeg", "leftLowerLeg");
    map.insert("LeftFoot", "leftFoot");
    map.insert("LeftToeBase", "leftToes");
    map.insert("RightUpLeg", "rightUpperLeg");
    map.insert("RightLeg", "rightLowerLeg");
    map.insert("RightFoot", "rightFoot");
    map.insert("RightToeBase", "rightToes");

    map
}
