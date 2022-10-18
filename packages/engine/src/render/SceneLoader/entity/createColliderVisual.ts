import {
  BoxGeometry,
  CylinderGeometry,
  Group,
  Mesh,
  MeshBasicMaterial,
  Quaternion,
  SphereGeometry,
  Vector3,
} from "three";

import { SceneMap } from "../types";
import { removeColliderVisual } from "./removeColliderVisual";

const wireframeMaterial = new MeshBasicMaterial({
  color: 0x000000,
  wireframe: true,
});

const tempVector3 = new Vector3();
const tempQuaternion = new Quaternion();

export function createColliderVisual(
  entityId: string,
  map: SceneMap,
  visuals: Group
) {
  const entity = map.entities.get(entityId);
  if (!entity) throw new Error("Entity not found");

  // Remove previous collider
  removeColliderVisual(entityId, map);

  // Create new collider
  let collider: Mesh | null = null;
  switch (entity.collider?.type) {
    case "box": {
      collider = new Mesh(
        new BoxGeometry(...entity.collider.size),
        wireframeMaterial
      );
      break;
    }

    case "sphere": {
      collider = new Mesh(
        new SphereGeometry(entity.collider.radius),
        wireframeMaterial
      );
      break;
    }

    case "cylinder": {
      collider = new Mesh(
        new CylinderGeometry(
          entity.collider.radius,
          entity.collider.radius,
          entity.collider.height,
          32
        ),
        wireframeMaterial
      );
      break;
    }
  }

  if (collider) {
    const object = map.objects.get(entityId);
    if (!object) throw new Error("Object not found");

    // Add new collider
    map.colliders.set(entityId, collider);
    visuals.add(collider);

    // Update collider position
    const globalPosition = object.getWorldPosition(tempVector3);
    const globalQuaternion = object.getWorldQuaternion(tempQuaternion);

    collider.position.copy(globalPosition);
    collider.quaternion.copy(globalQuaternion);
  }
}
