import { Mesh } from "three";

import { SceneMap } from "../types";

export function updateMeshMaterial(meshId: string, map: SceneMap) {
  const mesh = map.meshes.get(meshId);

  // Update material according to certain mesh properties
  // TODO: Create a new material if the mesh needs to modify it
  if (mesh?.type === "Primitives") {
    mesh.primitives.forEach((primitive) => {
      const object = map.objects.get(primitive.id);
      if (!object) throw new Error("Object not found");
      if (!(object instanceof Mesh)) throw new Error("Object is not a mesh");

      // Occlusion map needs a second set of UVs
      if (
        object.material.aoMap &&
        object.geometry.attributes.uv &&
        !object.geometry.attributes.uv2
      ) {
        object.geometry.setAttribute("uv2", object.geometry.attributes.uv);
      }

      // Enable flat shading if no normal attribute
      if (!object.geometry.attributes.normal)
        object.material.flatShading = true;

      // Enable vertex colors if color attribute
      if (object.geometry.attributes.color) object.material.vertexColors = true;

      // If three.js needs to generate tangents, flip normal map y
      if (!object.geometry.attributes.tangent) {
        const normalScale = Math.abs(object.material.normalScale.y);
        object.material.normalScale.y = -normalScale;
      }

      object.material.needsUpdate = true;
    });
  }
}
