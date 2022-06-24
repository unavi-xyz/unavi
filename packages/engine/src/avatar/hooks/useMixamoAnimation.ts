import { VRM } from "@pixiv/three-vrm";
import { useFrame } from "@react-three/fiber";
import { Group } from "three";

import { HumanBone } from "../types";
import { useMixamoBone } from "./useMixamoBone";

//takes a fbx animation from mixamo and applies it to a vrm model
export function useMixamoAnimation(fbx?: Group, vrm?: VRM) {
  const hips = useMixamoBone(fbx, "mixamorigHips", vrm, HumanBone.Hips);
  useMixamoBone(fbx, "mixamorigSpine1", vrm, HumanBone.Spine);
  useMixamoBone(fbx, "mixamorigSpine2", vrm, HumanBone.Chest);
  useMixamoBone(fbx, "mixamorigNeck", vrm, HumanBone.Neck);
  useMixamoBone(fbx, "mixamorigHead", vrm, HumanBone.Head);
  useMixamoBone(fbx, "mixamorigLeftArm", vrm, HumanBone.LeftUpperArm);
  useMixamoBone(fbx, "mixamorigLeftForeArm", vrm, HumanBone.LeftLowerArm);
  useMixamoBone(fbx, "mixamorigLeftHand", vrm, HumanBone.LeftHand);
  useMixamoBone(fbx, "mixamorigLeftUpLeg", vrm, HumanBone.LeftUpperLeg);
  useMixamoBone(fbx, "mixamorigLeftLeg", vrm, HumanBone.LeftLowerLeg);
  useMixamoBone(fbx, "mixamorigLeftFoot", vrm, HumanBone.LeftFoot);
  useMixamoBone(fbx, "mixamorigRightArm", vrm, HumanBone.RightUpperArm);
  useMixamoBone(fbx, "mixamorigRightForeArm", vrm, HumanBone.RightLowerArm);
  useMixamoBone(fbx, "mixamorigRightHand", vrm, HumanBone.RightHand);
  useMixamoBone(fbx, "mixamorigRightUpLeg", vrm, HumanBone.RightUpperLeg);
  useMixamoBone(fbx, "mixamorigRightLeg", vrm, HumanBone.RightLowerLeg);
  useMixamoBone(fbx, "mixamorigRightFoot", vrm, HumanBone.RightFoot);

  useFrame(() => {
    if (!hips) return;
    hips.position.set(-100, 100, 0);
  });
}
