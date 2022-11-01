import { Mesh } from "three";

import { SceneMap } from "../types";

export function updateMeshMaterial(nodeId: string, map: SceneMap) {
  const node = map.nodes.get(nodeId);
  if (!node) throw new Error("Node not found");

  const mesh = node.meshId ? map.meshes.get(node.meshId) : null;

  // Update material according to certain mesh properties
  // TODO: Create a new material if the mesh needs to modify it
  if (mesh?.type === "Primitives") {
    const meshObject = map.objects.get(nodeId);
    if (!meshObject) throw new Error("Mesh not found");
    if (!(meshObject instanceof Mesh)) throw new Error("Object is not a mesh");

    // Occlusion map needs a second set of UVs
    if (
      meshObject.material.aoMap &&
      meshObject.geometry.attributes.uv &&
      !meshObject.geometry.attributes.uv2
    ) {
      meshObject.geometry.setAttribute(
        "uv2",
        meshObject.geometry.attributes.uv
      );
    }

    // Enable flat shading if no normal attribute
    if (!meshObject.geometry.attributes.normal)
      meshObject.material.flatShading = true;

    // Enable vertex colors if color attribute
    if (meshObject.geometry.attributes.color)
      meshObject.material.vertexColors = true;

    // If three.js needs to generate tangents, flip normal map y
    if (!meshObject.geometry.attributes.tangent) {
      const normalScale = Math.abs(meshObject.material.normalScale.y);
      meshObject.material.normalScale.y = -normalScale;
    }

    meshObject.material.needsUpdate = true;
  }
}
