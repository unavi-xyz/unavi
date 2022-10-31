import { Quaternion, Vector3 } from "three";

import { PostMessage, Quad } from "../../../types";
import { FromRenderMessage } from "../../types";
import { SceneMap } from "../types";
import { getChildren } from "./getChildren";

const tempVector3 = new Vector3();
const tempQuaternion = new Quaternion();

export function updateGlobalTransform(
  nodeId: string,
  map: SceneMap,
  postMessage: PostMessage<FromRenderMessage>
) {
  const node = map.nodes.get(nodeId);
  if (!node) throw new Error("Node not found");

  const object = map.objects.get(nodeId);
  if (!object) throw new Error("Object not found");

  const globalPosition = object.getWorldPosition(tempVector3);
  const globalQuaternion = object.getWorldQuaternion(tempQuaternion);

  const rotation: Quad = [
    globalQuaternion.x,
    globalQuaternion.y,
    globalQuaternion.z,
    globalQuaternion.w,
  ];

  // Update collider transform
  const collider = map.colliders.get(nodeId);
  if (collider) {
    collider.position.copy(globalPosition);
    collider.quaternion.copy(globalQuaternion);
  }

  postMessage({
    subject: "set_global_transform",
    data: {
      nodeId,
      position: globalPosition.toArray(),
      rotation,
    },
  });

  // Repeat for children
  const children = getChildren(nodeId, map);
  children.forEach((child) =>
    updateGlobalTransform(child.id, map, postMessage)
  );
}
