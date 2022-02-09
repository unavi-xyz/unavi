import { useEffect, useRef, useState } from "react";
import { useFrame, useThree } from "@react-three/fiber";
import { VRM, VRMSchema } from "@pixiv/three-vrm";
import { Group, Object3D, Vector3 } from "three";

import { visualizePoint } from "../visualizers";

export default function useMixamoBone(
  fbx: Group | undefined,
  fbxName: string,
  vrm: VRM | undefined,
  vrmName: VRMSchema.HumanoidBoneName,
  visualize = false
) {
  const [fbxBone, setFbxBone] = useState<Object3D>();
  const [vrmBone, setVrmBone] = useState<Object3D>();

  const { scene } = useThree();
  const visual = useRef(visualizePoint(scene, new Vector3(0, -1000, 0)));

  useEffect(() => {
    if (!fbx) return;
    const bone = fbx.getObjectByName(fbxName);
    setFbxBone(bone);
  }, [fbx]);

  useEffect(() => {
    const humanBones = vrm?.humanoid?.humanBones;
    if (!humanBones) return;

    const bone = humanBones[vrmName][0].node;
    setVrmBone(bone);
  }, [vrm]);

  useFrame(() => {
    if (!fbxBone || !vrmBone) return;

    if (visualize) fbxBone.getWorldPosition(visual.current.position);

    //copy the fbx rotation to the vrm model
    const fbxRot = fbxBone.rotation.clone();
    fbxRot.x *= -1;
    fbxRot.z *= -1;

    vrmBone.rotation.copy(fbxRot);
  });

  return fbxBone;
}
