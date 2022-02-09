import { useEffect, useState } from "react";
import { useFrame } from "@react-three/fiber";
import { VRM } from "@pixiv/three-vrm";

import { Object3D, Vector3 } from "three";

import { cos_sss } from "../math";

export default function useLimb(vrm: VRM, which = "right", targetRef) {
  const [hips, setHips] = useState<Object3D>();
  const [upperLeg, setUpperLeg] = useState<Object3D>();
  const [knee, setKnee] = useState<Object3D>();
  const [ankle, setAnkle] = useState<Object3D>();

  useEffect(() => {
    if (!vrm?.humanoid) return;
    const bones = vrm.humanoid.humanBones;

    setHips(bones.hips[bones.hips.length - 1].node);
    if (which === "right") {
      setUpperLeg(bones.rightUpperLeg[bones.rightUpperLeg.length - 1].node);
      setKnee(bones.rightLowerLeg[bones.rightLowerLeg.length - 1].node);
      setAnkle(bones.rightFoot[bones.rightFoot.length - 1].node);
    } else {
      setUpperLeg(bones.leftUpperLeg[bones.leftUpperLeg.length - 1].node);
      setKnee(bones.leftLowerLeg[bones.leftLowerLeg.length - 1].node);
      setAnkle(bones.leftFoot[bones.leftFoot.length - 1].node);
    }
  }, [vrm, which]);

  useFrame(() => {
    const target = targetRef.current;
    if (!hips || !upperLeg || !knee || !ankle || !target) return;

    const upperLeg_pos = upperLeg.getWorldPosition(new Vector3());
    const knee_pos = knee.getWorldPosition(new Vector3());
    const ankle_pos = ankle.getWorldPosition(new Vector3());

    const len_upperLeg_target = upperLeg_pos.distanceTo(target);
    const len_upperLeg_knee = upperLeg_pos.distanceTo(knee_pos);
    const len_knee_ankle = knee_pos.distanceTo(ankle_pos);

    const len_leg = len_upperLeg_knee + len_knee_ankle;

    const sign = Math.sign(upperLeg.position.x);

    const isInFront = hips.position.z > hips.worldToLocal(target.clone()).z;

    if (len_upperLeg_target > len_leg) {
      //extend leg straight

      upperLeg.lookAt(target);
      upperLeg.rotateX(Math.PI / -2);
      if (isInFront) upperLeg.rotateY(Math.PI);

      knee.rotation.x = 0;
      knee.rotation.y = 0;
      knee.rotation.z = 0;
    } else {
      //bend the knee

      //calculate the angle the upper leg should rotate
      //do this by forming a triangle using the lengths of the upper leg, knee, and ankle
      //then use the law of cos sss equation to find the angle it forms

      const upperLegAngle =
        cos_sss(len_upperLeg_target, len_upperLeg_knee, len_knee_ankle) *
        -1 *
        sign;

      //get the point the knee should be at by rotating by this angle
      const knee_point = target
        .clone()
        .sub(upperLeg_pos)
        .applyAxisAngle(new Vector3(-sign, 0, 0), upperLegAngle)
        .add(upperLeg_pos);

      //turn the bones
      upperLeg.lookAt(knee_point);
      upperLeg.rotateX(Math.PI / -2);
      if (isInFront) upperLeg.rotateY(Math.PI);

      knee.lookAt(target);
      knee.rotateX(Math.PI / -2);

      if (knee.rotation.x > 0) {
        knee.rotateY(Math.PI);
      }
    }

    ankle.rotation.x = 0;
    ankle.rotation.y = 0;
    ankle.rotation.z = 0;
  });
}
