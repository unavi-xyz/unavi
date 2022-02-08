import { useEffect, useRef } from "react";
import { VRM, VRMSchema } from "@pixiv/three-vrm";
import { useFrame } from "@react-three/fiber";
import { AnimationMixer, Group } from "three";

import useMixamoBone from "./useMixamoBone";

const HumanBone = { ...VRMSchema.HumanoidBoneName };

//takes a fbx animation from mixamo and applies it to a vrm model
export default function useMixamoAnimation(fbx?: Group, vrm?: VRM) {
  useMixamoBone(fbx, "mixamorigHips", vrm, HumanBone.Hips);
  useMixamoBone(fbx, "mixamorigSpine1", vrm, HumanBone.Spine);
  useMixamoBone(fbx, "mixamorigSpine2", vrm, HumanBone.Chest);
  useMixamoBone(fbx, "mixamorigNeck", vrm, HumanBone.Neck);
  useMixamoBone(fbx, "mixamorigHead", vrm, HumanBone.Head);
  useMixamoBone(fbx, "mixamorigLeftShoulder", vrm, HumanBone.LeftShoulder);
  useMixamoBone(fbx, "mixamorigLeftArm", vrm, HumanBone.LeftUpperArm);
  useMixamoBone(fbx, "mixamorigLeftForeArm", vrm, HumanBone.LeftLowerLeg);
  useMixamoBone(fbx, "mixamorigLeftHand", vrm, HumanBone.LeftHand);
  useMixamoBone(fbx, "mixamorigLeftUpLeg", vrm, HumanBone.LeftUpperLeg);
  useMixamoBone(fbx, "mixamorigLeftLeg", vrm, HumanBone.LeftLowerLeg);
  useMixamoBone(fbx, "mixamorigLeftFoot", vrm, HumanBone.LeftFoot);
  useMixamoBone(fbx, "mixamorigRightShoulder", vrm, HumanBone.RightShoulder);
  useMixamoBone(fbx, "mixamorigRightArm", vrm, HumanBone.RightUpperArm);
  useMixamoBone(fbx, "mixamorigRightForeArm", vrm, HumanBone.RightLowerArm);
  useMixamoBone(fbx, "mixamorigRightHand", vrm, HumanBone.RightHand);
  useMixamoBone(fbx, "mixamorigRightUpLeg", vrm, HumanBone.RightUpperLeg);
  useMixamoBone(fbx, "mixamorigRightLeg", vrm, HumanBone.RightLowerLeg);
  useMixamoBone(fbx, "mixamorigRightFoot", vrm, HumanBone.RightFoot);

  const mixer = useRef<AnimationMixer>();

  useEffect(() => {
    if (!fbx) return;

    const scale = 0.01;
    fbx.scale.set(scale, scale, scale);
    fbx.rotation.y = Math.PI;

    mixer.current = new AnimationMixer(fbx);
    const action = mixer.current.clipAction(fbx.animations[0]);
    action.play();
  }, [fbx]);

  useFrame(({ clock }) => {
    const delta = Math.max(clock.getDelta(), 0.0001) * 60;
    mixer.current?.update(delta);

    if (!fbx) return;

    const hips = fbx.getObjectByName("mixamorigHips");
    hips?.position.set(-100, 100, 0);
  });
}
