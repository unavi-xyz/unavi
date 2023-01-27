import { VRM } from "@pixiv/three-vrm";
import {
  AnimationClip,
  KeyframeTrack,
  Quaternion,
  QuaternionKeyframeTrack,
  Vector3,
  VectorKeyframeTrack,
} from "three";
import { FBXLoader } from "three/examples/jsm/loaders/FBXLoader";

/*
 * Loads a Mixamo animation, converts for vrm use, and returns it.
 * Heavily based on {@link https://github.com/pixiv/three-vrm/blob/dev/packages/three-vrm-core/examples/humanoidAnimation/loadMixamoAnimation.js}
 */
export async function loadMixamoAnimation(
  animationsPath: string,
  vrm: VRM
): Promise<AnimationClip[]> {
  const loader = new FBXLoader();

  const restRotationInverse = new Quaternion();
  const parentRestWorldRotation = new Quaternion();
  const quat = new Quaternion();
  const vec3 = new Vector3();

  const fbx = await loader.loadAsync(animationsPath);

  const animationHips = fbx.getObjectByName("mixamorigHips");
  if (!animationHips) throw new Error("No animation hips");

  const vrmHips = vrm.humanoid.getNormalizedBoneNode("hips");
  if (!vrmHips) throw new Error("No VRM hips");

  // Adjust with reference to hips height
  const vrmHipsY = vrmHips.getWorldPosition(vec3).y;
  const vrmRootY = vrm.scene.getWorldPosition(vec3).y;
  const vrmHipsHeight = Math.abs(vrmHipsY - vrmRootY);
  const animationHipsHeight = animationHips.position.y;
  const hipsPositionScale = vrmHipsHeight / animationHipsHeight;

  const clips = fbx.animations.map((clip) => {
    const tracks: KeyframeTrack[] = [];

    clip.tracks.forEach((track) => {
      const splitTrack = track.name.split(".");

      const mixamoRigName = splitTrack[0] as keyof typeof mixamoRigMap;
      const vrmBoneName = mixamoRigMap[mixamoRigName];
      if (!vrmBoneName) return;

      const vrmBone = vrm.humanoid.getNormalizedBoneNode(vrmBoneName);
      if (!vrmBone) return;

      const mixamoRigNode = fbx.getObjectByName(mixamoRigName);
      if (!mixamoRigNode) throw new Error("No mixamo rig node");
      if (!mixamoRigNode.parent) throw new Error("No mixamo rig parent");

      const propertyName = splitTrack[1];
      if (!propertyName) throw new Error("No property name");

      // Store rotations of rest-pose.
      mixamoRigNode.getWorldQuaternion(restRotationInverse).invert();
      mixamoRigNode.parent.getWorldQuaternion(parentRestWorldRotation);

      if (track instanceof QuaternionKeyframeTrack) {
        // Retarget rotation of mixamoRig to NormalizedBone.
        for (let i = 0; i < track.values.length; i += 4) {
          const flatQuaternion = track.values.slice(i, i + 4);

          quat.fromArray(flatQuaternion);

          quat.premultiply(parentRestWorldRotation).multiply(restRotationInverse);

          quat.toArray(flatQuaternion);

          flatQuaternion.forEach((v, index) => {
            track.values[index + i] = v;
          });
        }

        tracks.push(
          new QuaternionKeyframeTrack(
            `${vrmBone.name}.${propertyName}`,
            track.times as any,
            track.values.map((v, i) =>
              vrm.meta?.metaVersion === "0" && i % 2 === 0 ? -v : v
            ) as any
          )
        );
      } else if (track instanceof VectorKeyframeTrack) {
        const value = track.values.map(
          (v, i) => (vrm.meta?.metaVersion === "0" && i % 3 !== 1 ? -v : v) * hipsPositionScale
        );
        tracks.push(
          new VectorKeyframeTrack(
            `${vrmBone.name}.${propertyName}`,
            track.times as any,
            value as any
          )
        );
      }
    });

    return new AnimationClip(clip.name, clip.duration, tracks);
  });

  return clips;
}

const mixamoRigMap = {
  mixamorigHips: "hips",
  mixamorigSpine: "spine",
  mixamorigSpine1: "chest",
  mixamorigSpine2: "upperChest",
  mixamorigNeck: "neck",
  mixamorigHead: "head",
  mixamorigLeftShoulder: "leftShoulder",
  mixamorigLeftArm: "leftUpperArm",
  mixamorigLeftForeArm: "leftLowerArm",
  mixamorigLeftHand: "leftHand",
  mixamorigLeftHandThumb1: "leftThumbMetacarpal",
  mixamorigLeftHandThumb2: "leftThumbProximal",
  mixamorigLeftHandThumb3: "leftThumbDistal",
  mixamorigLeftHandIndex1: "leftIndexProximal",
  mixamorigLeftHandIndex2: "leftIndexIntermediate",
  mixamorigLeftHandIndex3: "leftIndexDistal",
  mixamorigLeftHandMiddle1: "leftMiddleProximal",
  mixamorigLeftHandMiddle2: "leftMiddleIntermediate",
  mixamorigLeftHandMiddle3: "leftMiddleDistal",
  mixamorigLeftHandRing1: "leftRingProximal",
  mixamorigLeftHandRing2: "leftRingIntermediate",
  mixamorigLeftHandRing3: "leftRingDistal",
  mixamorigLeftHandPinky1: "leftLittleProximal",
  mixamorigLeftHandPinky2: "leftLittleIntermediate",
  mixamorigLeftHandPinky3: "leftLittleDistal",
  mixamorigRightShoulder: "rightShoulder",
  mixamorigRightArm: "rightUpperArm",
  mixamorigRightForeArm: "rightLowerArm",
  mixamorigRightHand: "rightHand",
  mixamorigRightHandPinky1: "rightLittleProximal",
  mixamorigRightHandPinky2: "rightLittleIntermediate",
  mixamorigRightHandPinky3: "rightLittleDistal",
  mixamorigRightHandRing1: "rightRingProximal",
  mixamorigRightHandRing2: "rightRingIntermediate",
  mixamorigRightHandRing3: "rightRingDistal",
  mixamorigRightHandMiddle1: "rightMiddleProximal",
  mixamorigRightHandMiddle2: "rightMiddleIntermediate",
  mixamorigRightHandMiddle3: "rightMiddleDistal",
  mixamorigRightHandIndex1: "rightIndexProximal",
  mixamorigRightHandIndex2: "rightIndexIntermediate",
  mixamorigRightHandIndex3: "rightIndexDistal",
  mixamorigRightHandThumb1: "rightThumbMetacarpal",
  mixamorigRightHandThumb2: "rightThumbProximal",
  mixamorigRightHandThumb3: "rightThumbDistal",
  mixamorigLeftUpLeg: "leftUpperLeg",
  mixamorigLeftLeg: "leftLowerLeg",
  mixamorigLeftFoot: "leftFoot",
  mixamorigLeftToeBase: "leftToes",
  mixamorigRightUpLeg: "rightUpperLeg",
  mixamorigRightLeg: "rightLowerLeg",
  mixamorigRightFoot: "rightFoot",
  mixamorigRightToeBase: "rightToes",
} as const;
