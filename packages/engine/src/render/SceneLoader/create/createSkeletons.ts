import { Bone, Matrix4, Skeleton, SkinnedMesh } from "three";

import { RenderScene } from "../../RenderScene";
import { SceneMap } from "../types";

export function createSkeletons(scene: RenderScene, map: SceneMap) {
  Object.values(scene.entities).forEach((e) => {
    const isSkin = e.mesh?.type === "Primitive" && e.mesh.skin !== null;
    if (!isSkin) return;

    const skinObject = map.objects.get(e.id);
    if (!skinObject) return;

    // Create skeleton for skin and all child skins
    skinObject.traverse((child) => {
      if (!(child instanceof SkinnedMesh)) return;

      const bones: Bone[] = [];
      const boneInverses: Matrix4[] = [];

      if (e.mesh?.type !== "Primitive") throw new Error("Mesh not primitive");
      if (!e.mesh.skin?.inverseBindMatricesId)
        throw new Error("No inverse bind matrices");

      const inverseBindMatrices =
        scene.accessors[e.mesh.skin.inverseBindMatricesId].array;

      e.mesh.skin.jointIds.forEach((jointId, i) => {
        const bone = map.objects.get(jointId);
        if (!(bone instanceof Bone)) throw new Error("Bone not found");

        const matrix = new Matrix4();
        matrix.fromArray(inverseBindMatrices, i * 16);

        bones.push(bone);
        boneInverses.push(matrix);
      });

      child.bind(new Skeleton(bones, boneInverses), child.matrixWorld);
    });
  });
}
