import {
  BoxBufferGeometry,
  CylinderBufferGeometry,
  Group,
  Mesh,
  MeshBasicMaterial,
  SphereBufferGeometry,
} from "three";

import { RenderScene } from "../../RenderScene";
import { removeColliderVisual } from "../remove/removeColliderVisual";
import { SceneMap } from "../types";

const wireframeMaterial = new MeshBasicMaterial({
  color: 0x000000,
  wireframe: true,
});

export function createColliderVisual(
  entityId: string,
  map: SceneMap,
  scene: RenderScene,
  visuals: Group
) {
  const entity = scene.entities[entityId];
  if (!entity) throw new Error("Entity not found");

  // Remove previous collider
  removeColliderVisual(entityId, map);

  // Create new collider
  let collider: Mesh | null = null;
  switch (entity.collider?.type) {
    case "Box":
      collider = new Mesh(
        new BoxBufferGeometry(...entity.collider.size),
        wireframeMaterial
      );
      break;
    case "Sphere":
      collider = new Mesh(
        new SphereBufferGeometry(entity.collider.radius),
        wireframeMaterial
      );
      break;
    case "Cylinder":
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

  if (collider) {
    const object = map.objects.get(entityId);
    if (!object) throw new Error("Object not found");

    // Add new collider
    map.colliders.set(entityId, collider);
    visuals.add(collider);

    // Subscribe to global transform changes
    entity.globalPosition$.subscribe({
      next: (position) => {
        if (collider) collider.position.fromArray(position);
      },
    });

    entity.globalQuaternion$.subscribe({
      next: (quaternion) => {
        if (collider) collider.quaternion.fromArray(quaternion);
      },
    });
  }
}
