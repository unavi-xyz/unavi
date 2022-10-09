import {
  BoxBufferGeometry,
  CylinderBufferGeometry,
  Group,
  Mesh,
  MeshBasicMaterial,
  Quaternion,
  SphereBufferGeometry,
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
        new BoxBufferGeometry(...entity.collider.size),
        wireframeMaterial
      );
      break;
    }

    case "sphere": {
      collider = new Mesh(
        new SphereBufferGeometry(entity.collider.radius),
        wireframeMaterial
      );
      break;
    }

    case "cylinder": {
      collider = new Mesh(
        new CylinderBufferGeometry(
          entity.collider.radius,
          entity.collider.radius,
          entity.collider.height
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
