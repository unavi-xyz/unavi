import { Mesh } from "three";

import { SceneMap } from "../types";

export function updateMeshMaterial(meshId: string, map: SceneMap) {
  const mesh = map.meshes.get(meshId);

  // Update material according to certain mesh properties
  if (mesh?.type === "Primitives") {
    mesh.primitives.forEach((primitive) => {
      if (!primitive.materialId) return;

      const object = map.objects.get(primitive.id);
      if (!object) throw new Error("Object not found");
      if (!(object instanceof Mesh)) throw new Error("Object is not a mesh");

      const useUv2 =
        object.material.aoMap && object.geometry.attributes.uv && !object.geometry.attributes.uv2;
      const useFlatShading = !object.geometry.attributes.normal;
      const useVertexColors = Boolean(object.geometry.attributes.color);
      const useFlipTangents = !object.geometry.attributes.tangent;

      const cacheKey = `${primitive.materialId}-${useUv2}-${useFlatShading}-${useVertexColors}-${useFlipTangents}`;

      // Create a new material variant if we need to make any changes
      function setVariant() {
        if (!(object instanceof Mesh)) throw new Error("Object is not a mesh");
        if (!primitive.materialId) throw new Error("Material not found");

        // Check if we already have a variant of this material
        const variant = map.materialVariants.get(cacheKey);
        if (variant) {
          object.material = variant;
          return;
        }

        const material = map.materials.get(primitive.materialId);
        if (!material) throw new Error("Material not found");

        // Clone material and add it to the cache
        const newMaterial = material.clone();
        object.material = newMaterial;

        map.materialVariants.set(cacheKey, newMaterial);
      }

      if (useUv2) {
        setVariant();
        object.geometry.setAttribute("uv2", object.geometry.attributes.uv);
      }

      if (useFlatShading) {
        setVariant();
        object.material.flatShading = true;
      }

      if (useVertexColors) {
        setVariant();
        object.material.vertexColors = true;
      }

      if (useFlipTangents) {
        setVariant();
        const normalScale = Math.abs(object.material.normalScale.y);
        object.material.normalScale.y = -normalScale;
      }

      object.material.needsUpdate = true;
    });
  }
}
