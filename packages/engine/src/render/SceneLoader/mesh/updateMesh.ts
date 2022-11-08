import { Mesh, Quaternion, Vector3 } from "three";

import { MeshJSON } from "../../../scene";
import { PostMessage } from "../../../types";
import { FromRenderMessage } from "../../types";
import { defaultMaterial } from "../constants";
import { createColliderVisual } from "../node/createColliderVisual";
import { SceneMap } from "../types";
import { createObject } from "./createObject";
import { updateMeshMaterial } from "./updateMeshMaterial";

export function updateMesh(
  meshId: string,
  data: Partial<MeshJSON>,
  map: SceneMap,
  postMessage: PostMessage<FromRenderMessage>
) {
  const mesh = map.meshes.get(meshId);
  if (!mesh) throw new Error("Mesh not found");

  Object.assign(mesh, data);

  const oldObject = map.objects.get(meshId);
  const position = oldObject ? oldObject.position.clone() : new Vector3();
  const quaternion = oldObject
    ? oldObject.quaternion.clone()
    : new Quaternion();
  const scale = oldObject ? oldObject.scale.clone() : new Vector3(1, 1, 1);

  createObject(meshId, map, postMessage);

  const object = map.objects.get(meshId);
  if (!object) throw new Error("Error creating object");

  object.position.copy(position);
  object.quaternion.copy(quaternion);
  object.scale.copy(scale);

  if (object instanceof Mesh) {
    const material = mesh.materialId
      ? map.materials.get(mesh.materialId)
      : defaultMaterial;
    if (!material) throw new Error("Material not found");

    object.material = material;
  }

  updateMeshMaterial(meshId, map);

  // Update auto collider visuals for nodes that use this mesh
  for (const node of map.nodes.values()) {
    if (node.meshId === meshId && node.collider) {
      createColliderVisual(node.id, map, postMessage);
    }
  }
}
