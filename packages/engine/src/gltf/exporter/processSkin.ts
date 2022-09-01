import {
  BufferAttribute,
  InterleavedBufferAttribute,
  Matrix4,
  Object3D,
  SkinnedMesh,
} from "three";

import { GLTF, Skin } from "../schemaTypes";

export function processSkin(
  skin: SkinnedMesh,
  json: GLTF,
  processNode: (node: Object3D) => number,
  processAccessor: (
    accessor: BufferAttribute | InterleavedBufferAttribute
  ) => number | null
) {
  const inverseBindMatrices = new Float32Array(skin.skeleton.bones.length * 16);
  const tempBoneMatrix = new Matrix4();

  const joints = skin.skeleton.bones.map((bone, i) => {
    // Inverse bind matrix
    tempBoneMatrix.copy(skin.skeleton.boneInverses[i]);
    tempBoneMatrix
      .multiply(skin.bindMatrix)
      .toArray(inverseBindMatrices, i * 16);

    // Joint
    const boneIndex = processNode(bone);
    return boneIndex;
  });

  const rootIndex = processNode(skin.skeleton.bones[0]);
  const inverseBindMatricesIndex = processAccessor(
    new BufferAttribute(inverseBindMatrices, 16)
  );
  if (inverseBindMatricesIndex == null)
    throw new Error("Invalid inverse bind matrices");

  const skinDef: Skin = {
    joints,
    skeleton: rootIndex,
    inverseBindMatrices: inverseBindMatricesIndex,
  };

  if (!json.skins) json.skins = [];
  const index = json.skins.push(skinDef) - 1;
  skinDef.name = skin.name || `skin_${index}`;

  // Set node skin
  const skinNodeIndex = processNode(skin);
  if (!json.nodes) throw new Error("No nodes");

  const skinNodeDef = json.nodes[skinNodeIndex];
  skinNodeDef.skin = index;

  return index;
}
