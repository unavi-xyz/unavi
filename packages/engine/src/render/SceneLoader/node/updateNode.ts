import { Group, Mesh, Quaternion, Vector3 } from "three";

import { NodeJSON } from "../../../scene";
import { PostMessage } from "../../../types";
import { FromRenderMessage } from "../../types";
import { defaultMaterial } from "../constants";
import { SceneMap } from "../types";
import { updateGlobalTransform } from "../utils/updateGlobalTransform";
import { createColliderVisual } from "./createColliderVisual";
import { createObject } from "./createObject";
import { updateMeshMaterial } from "./updateMeshMaterial";

const tempVector = new Vector3();
const tempQuaternion = new Quaternion();
const tempQuaternion2 = new Quaternion();

export function updateNode(
  nodeId: string,
  data: Partial<NodeJSON>,
  map: SceneMap,
  visuals: Group,
  postMessage: PostMessage<FromRenderMessage>
) {
  const node = map.nodes.get(nodeId);
  if (!node) throw new Error("Node not found");

  const newNode = { ...node, ...data };
  map.nodes.set(nodeId, newNode);

  // Update object
  if (data.mesh !== undefined) {
    const oldObject = map.objects.get(nodeId);
    const position = oldObject ? oldObject.position.clone() : new Vector3();
    const quaternion = oldObject
      ? oldObject.quaternion.clone()
      : new Quaternion();
    const scale = oldObject ? oldObject.scale.clone() : new Vector3(1, 1, 1);

    createObject(newNode, map, visuals, postMessage);

    const object = map.objects.get(nodeId);
    if (!object) throw new Error("Object not found");

    object.position.copy(position);
    object.quaternion.copy(quaternion);
    object.scale.copy(scale);
  }

  const object = map.objects.get(nodeId);
  if (!object) throw new Error("Object not found");

  // Update parent
  if (data.parentId !== undefined) {
    const parentObject = map.objects.get(data.parentId);
    if (!parentObject) throw new Error("Parent not found");

    // Save object transform
    const position = object.getWorldPosition(tempVector);
    const quaternion = object.getWorldQuaternion(tempQuaternion);

    // Add to parent
    parentObject.add(object);

    // Restore object transform
    const inverseParentRotation = parentObject
      .getWorldQuaternion(tempQuaternion2)
      .invert();
    object.position.copy(parentObject.worldToLocal(position));
    object.quaternion.multiplyQuaternions(quaternion, inverseParentRotation);

    // Update global transform
    updateGlobalTransform(nodeId, map, postMessage);
  }

  // Update name
  if (data.name !== undefined) object.name = data.name || nodeId;

  // Update transform
  if (data.position !== undefined) object.position.fromArray(data.position);
  if (data.rotation !== undefined) object.quaternion.fromArray(data.rotation);
  if (data.scale !== undefined) object.scale.fromArray(data.scale);

  if (data.position !== undefined || data.rotation !== undefined) {
    updateGlobalTransform(nodeId, map, postMessage);
  }

  // Update material
  if (data.materialId !== undefined) {
    if (!(object instanceof Mesh)) {
      if (data.materialId !== null) throw new Error("Object is not a mesh");
    } else {
      const material = data.materialId
        ? map.materials.get(data.materialId)
        : defaultMaterial;
      if (!material) throw new Error("Material not found");

      object.material = material;

      updateMeshMaterial(nodeId, node.mesh, map);
    }
  }

  // Update collider visual
  if (data.mesh || data.collider)
    createColliderVisual(nodeId, map, visuals, postMessage);
}
