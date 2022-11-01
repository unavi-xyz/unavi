import { Group, Quaternion, Vector3 } from "three";

import { NodeJSON } from "../../../scene";
import { PostMessage } from "../../../types";
import { FromRenderMessage } from "../../types";
import { SceneMap } from "../types";
import { updateGlobalTransform } from "../utils/updateGlobalTransform";
import { createColliderVisual } from "./createColliderVisual";

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

  let object = map.objects.get(nodeId);
  if (!object) {
    object = new Group();
    map.objects.set(nodeId, object);
  }

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

  // Mesh
  if (data.meshId !== undefined) {
    let meshGroup = map.objects.get(meshKey(nodeId));
    if (!meshGroup) {
      meshGroup = new Group();
      map.objects.set(meshKey(nodeId), meshGroup);
      object.add(meshGroup);
    }

    if (data.meshId) {
      // Add mesh as child
      const meshObject = map.objects.get(data.meshId);
      if (!meshObject) throw new Error("Mesh not found");

      meshGroup.add(meshObject);
    } else {
      // Remove mesh
      meshGroup.remove(...meshGroup.children);
    }
  }

  // Update collider visual
  if (data.meshId !== undefined || data.collider !== undefined)
    createColliderVisual(nodeId, map, visuals, postMessage);
}

function meshKey(nodeId: string) {
  return `mesh-${nodeId}`;
}
