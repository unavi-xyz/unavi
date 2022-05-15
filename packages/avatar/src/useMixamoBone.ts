import { VRM, VRMSchema } from "@pixiv/three-vrm";
import { useFrame } from "@react-three/fiber";
import { useEffect, useState } from "react";
import { Group, Object3D } from "three";

export function useMixamoBone(
  fbx: Group | undefined,
  fbxName: string,
  vrm: VRM | undefined,
  vrmName: VRMSchema.HumanoidBoneName
) {
  const [fbxBone, setFbxBone] = useState<Object3D>();
  const [vrmBone, setVrmBone] = useState<Object3D>();

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

    //copy the fbx rotation to the vrm model
    const fbxRot = fbxBone.rotation.clone();
    fbxRot.x *= -1;
    fbxRot.z *= -1;

    vrmBone.rotation.copy(fbxRot);
  });

  return fbxBone;
}
