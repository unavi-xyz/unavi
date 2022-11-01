import { Bone, Matrix4, Skeleton, SkinnedMesh } from "three";

import { SceneMap } from "../types";

export function createSkeletons(map: SceneMap) {
  map.meshes.forEach((mesh) => {
    if (mesh.type !== "Primitives") return;

    mesh.primitives.forEach((primitive) => {
      if (!primitive.skin) return;

      const skinObject = map.objects.get(primitive.id);
      if (!skinObject) return;

      // Create skeleton for skin and all child skins
      skinObject.traverse((child) => {
        if (!primitive.skin) return;
        if (!(child instanceof SkinnedMesh)) return;

        const bones: Bone[] = [];
        const boneInverses: Matrix4[] = [];

        if (!primitive.skin.inverseBindMatricesId)
          throw new Error("No inverse bind matrices");

        const inverseBindMatrices = map.accessors.get(
          primitive.skin.inverseBindMatricesId
        );
        if (!inverseBindMatrices) throw new Error("Accessor not found");

        primitive.skin.jointIds.forEach((jointId, i) => {
          const bone = map.objects.get(jointId);
          if (!(bone instanceof Bone)) throw new Error("Bone not found");

          const matrix = new Matrix4();
          matrix.fromArray(inverseBindMatrices.array, i * 16);

          bones.push(bone);
          boneInverses.push(matrix);
        });

        child.bind(new Skeleton(bones, boneInverses), child.matrixWorld);
      });
    });
  });
}
