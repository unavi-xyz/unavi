import { Bone, Group, Quaternion, Vector3 } from "three";

import { NodeJSON } from "../../../scene";
import { PostMessage } from "../../../types";
import { FromRenderMessage } from "../../types";
import { createSkeletons } from "../mesh/createSkeletons";
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
  postMessage: PostMessage<FromRenderMessage>
) {
  const node = map.nodes.get(nodeId);
  if (!node) throw new Error("Node not found");

  const newNode = { ...node, ...data };
  map.nodes.set(nodeId, newNode);

  let object = map.objects.get(nodeId);
  if (!object) {
    // Check if joint
    let isJoint = false;

    map.meshes.forEach((mesh) => {
      if (mesh?.type !== "Primitives") return;
      if (!mesh.primitives.some((p) => p.skin?.jointIds.includes(nodeId)))
        return;

      isJoint = true;
    });

    // Create object
    object = isJoint ? new Bone() : new Group();
    map.objects.set(nodeId, object);

    // Update skeletons
    if (isJoint) createSkeletons(map);
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
      const meshObject = map.objects.get(data.meshId);
      if (!meshObject) throw new Error("Mesh not found");

      // Add mesh to queue
      map.objectQueue.add(meshObject, meshGroup);
    } else {
      // Remove mesh
      meshGroup.remove(...meshGroup.children);
    }
  }

  // Update collider visual
  if (data.meshId !== undefined || data.collider !== undefined)
    createColliderVisual(nodeId, map, postMessage);
}

function meshKey(nodeId: string) {
  return `mesh-${nodeId}`;
}
