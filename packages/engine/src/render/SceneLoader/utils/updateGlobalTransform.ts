import { Quaternion, Vector3 } from "three";

import { PostMessage, Quad } from "../../../types";
import { FromRenderMessage } from "../../types";
import { SceneMap } from "../types";
import { getChildren } from "./getChildren";

const tempVector3 = new Vector3();
const tempQuaternion = new Quaternion();

export function updateGlobalTransform(
  entityId: string,
  map: SceneMap,
  postMessage: PostMessage<FromRenderMessage>
) {
  const object = map.objects.get(entityId);
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
  const collider = map.colliders.get(entityId);
  if (collider) {
    collider.position.copy(globalPosition);
    collider.quaternion.copy(globalQuaternion);
  }

  postMessage({
    subject: "set_global_transform",
    data: {
      entityId,
      position: globalPosition.toArray(),
      rotation,
    },
  });

  // Repeat for children
  const entity = map.entities.get(entityId);
  if (!entity) throw new Error("Entity not found");

  const children = getChildren(entityId, map);
  children.forEach((child) =>
    updateGlobalTransform(child.id, map, postMessage)
  );
}
