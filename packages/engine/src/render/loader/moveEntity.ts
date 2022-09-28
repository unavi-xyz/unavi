import { Quaternion, Vector3 } from "three";

import { RenderScene } from "../RenderScene";
import { SceneMap } from "./SceneMap";
import { updateGlobalTransform } from "./updateGlobalTransform";

const tempVector = new Vector3();
const tempQuaternion = new Quaternion();
const tempQuaternion2 = new Quaternion();

export function moveEntity(
  entityId: string,
  parentId: string,
  map: SceneMap,
  scene: RenderScene
) {
  const object = map.objects.get(entityId);
  if (!object) throw new Error(`Object not found: ${entityId}`);

  const parentObject = map.objects.get(parentId);
  if (!parentObject) throw new Error(`Parent not found: ${parentId}`);

  // Save object transform
  const position = object.getWorldPosition(tempVector);
  const quaternion = object.getWorldQuaternion(tempQuaternion);

  // Set parent
  parentObject.add(object);

  // Restore object transform
  const inverseParentRotation = parentObject
    .getWorldQuaternion(tempQuaternion2)
    .invert();
  object.position.copy(parentObject.worldToLocal(position));
  object.quaternion.multiplyQuaternions(quaternion, inverseParentRotation);

  // Subscribe to global transform changes
  const parent = scene.entities[parentId];

  parent.globalPosition$.subscribe({
    next: () => {
      updateGlobalTransform(entityId, map, scene);
    },
  });

  parent.globalQuaternion$.subscribe({
    next: () => {
      updateGlobalTransform(entityId, map, scene);
    },
  });
}
