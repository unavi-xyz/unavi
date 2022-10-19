import { Mesh } from "three";

import { SceneMap } from "../types";

export function removeColliderVisual(entityId: string, map: SceneMap) {
  const collider = map.colliders.get(entityId);
  if (!collider) return;

  map.colliders.delete(entityId);
  collider.removeFromParent();
  collider.traverse((child) => {
    if (child instanceof Mesh) child.geometry.dispose();
  });
}
