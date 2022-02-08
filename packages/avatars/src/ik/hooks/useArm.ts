import { useEffect, useState } from "react";
import { useFrame } from "@react-three/fiber";
import { VRM } from "@pixiv/three-vrm";
import { Object3D, Vector3 } from "three";

import { cos_sss } from "../math";

export default function useLimb(vrm: VRM, which = "right", targetRef) {
  const [shoulder, setShoulder] = useState<Object3D>();
  const [elbow, setElbow] = useState<Object3D>();
  const [wrist, setWrist] = useState<Object3D>();

  useEffect(() => {
    if (!vrm?.humanoid) return;
    const bones = vrm.humanoid.humanBones;

    if (which === "right") {
      setShoulder(bones.rightShoulder[bones.rightShoulder.length - 1].node);
      setElbow(bones.rightLowerArm[bones.rightLowerArm.length - 1].node);
      setWrist(bones.rightHand[bones.rightHand.length - 1].node);
    } else {
      setShoulder(bones.leftShoulder[bones.leftShoulder.length - 1].node);
      setElbow(bones.leftLowerArm[bones.leftLowerArm.length - 1].node);
      setWrist(bones.leftHand[bones.leftHand.length - 1].node);
    }
  }, [vrm, which]);

  useFrame(() => {
    const target = targetRef.current;
    if (!shoulder || !elbow || !wrist || !target) return;

    const shoulder_pos = shoulder.getWorldPosition(new Vector3());
    const elbow_pos = elbow.getWorldPosition(new Vector3());
    const wrist_pos = wrist.getWorldPosition(new Vector3());

    const len_shoulder_target = shoulder_pos.distanceTo(target);
    const len_shoulder_elbow = shoulder_pos.distanceTo(elbow_pos);
    const len_elbow_wrist = elbow_pos.distanceTo(wrist_pos);

    const len_arm = len_shoulder_elbow + len_elbow_wrist;

    const sign = Math.sign(shoulder.position.x);

    if (len_shoulder_target > len_arm) {
      //extend arm straight
      shoulder.lookAt(target);
      shoulder.rotateY((Math.PI / -2) * sign);

      elbow.lookAt(target);
      elbow.rotateY((Math.PI / -2) * sign);
    } else {
      //bend the elbow

      //calculate the angle the shoulder should rotate
      //do this by forming a triangle using the lengths of the shoulder, elbow, and wrist
      //then use the law of cos sss equation to find the angle it forms

      const shoulderAngle =
        cos_sss(len_shoulder_target, len_shoulder_elbow, len_elbow_wrist) *
        -1 *
        sign;

      //get the point the elbow should be at by rotating by this angle
      const elbow_point = target
        .clone()
        .sub(shoulder_pos)
        .applyAxisAngle(new Vector3(0, 1, 0), shoulderAngle)
        .add(shoulder_pos);

      //turn the bones
      shoulder.lookAt(elbow_point);
      shoulder.rotateY((Math.PI / -2) * sign);

      elbow.lookAt(target);
      elbow.rotateY((Math.PI / -2) * sign);
    }

    wrist.rotation.x = 0;
    wrist.rotation.y = 0;
    wrist.rotation.z = 0;
  });
}
