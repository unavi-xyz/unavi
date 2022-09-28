import { Mesh } from "three";

import { MeshJSON } from "../../scene";
import { RenderScene } from "../RenderScene";
import { createMeshGeometry } from "./createMeshGeometry";
import { SceneMap } from "./SceneMap";

export function updateMesh(
  entityId: string,
  json: MeshJSON,
  map: SceneMap,
  scene: RenderScene
) {
  const object = map.objects.get(entityId);
  if (!object) throw new Error(`Object not found: ${entityId}`);
  const isMesh = object instanceof Mesh;
  if (!isMesh) throw new Error("Object is not a mesh");

  object.geometry.dispose();
  object.geometry = createMeshGeometry(json, map, scene);

  switch (json.type) {
    case "Primitive":
    case "Skin":
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
      if (!object.geometry.attributes.tangent)
        object.material.normalScale.y *= -1;

      // Set weights
      object.updateMorphTargets();
      object.morphTargetInfluences = [...json.weights];
      break;
  }
}
