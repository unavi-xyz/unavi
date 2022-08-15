import { Bone, Group, Matrix4, Object3D, Skeleton, SkinnedMesh } from "three";

import { GLTF } from "../schemaTypes";
import { SkinResult } from "./loadSkin";

export async function loadNode(
  index: number,
  json: GLTF,
  boneIndexes: Set<number>,
  loadMesh: (index: number) => Promise<Object3D>,
  loadSkin: (index: number) => Promise<SkinResult>,
  _loadNode: (index: number) => Promise<Object3D>
): Promise<Object3D> {
  if (json.nodes === undefined) {
    throw new Error("No nodes found");
  }

  const nodeDef = json.nodes[index];
  const node = boneIndexes.has(index) ? new Bone() : new Group();

  // Transform
  if (nodeDef.matrix !== undefined) {
    const matrix = new Matrix4();
    matrix.fromArray(nodeDef.matrix);
    node.applyMatrix4(matrix);
  } else {
    if (nodeDef.translation) {
      node.position.fromArray(nodeDef.translation);
    }

    if (nodeDef.rotation) {
      node.quaternion.fromArray(nodeDef.rotation);
    }

    if (nodeDef.scale) {
      node.scale.fromArray(nodeDef.scale);
    }
  }

  // Mesh
  if (nodeDef.mesh !== undefined) {
    const mesh = await loadMesh(nodeDef.mesh);
    node.add(mesh);
  }

  // Skin
  if (nodeDef.skin !== undefined) {
    const skinEntry = await loadSkin(nodeDef.skin);

    const jointPromises = skinEntry.joints.map((joint) => _loadNode(joint));

    const joints = await Promise.all(jointPromises);

    node.traverse((child) => {
      if (!(child instanceof SkinnedMesh)) return;

      const bones: Bone[] = [];
      const boneInverses: Matrix4[] = [];

      joints.forEach((joint, i) => {
        if (!(joint instanceof Bone)) return;

        const matrix = new Matrix4();

        if (skinEntry.inverseBindMatrices !== null) {
          matrix.fromArray(skinEntry.inverseBindMatrices.array, i * 16);
        }

        bones.push(joint);
        boneInverses.push(matrix);
      });

      child.bind(new Skeleton(bones, boneInverses), child.matrixWorld);
    });
  }

  return node;
}
