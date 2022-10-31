import { Mesh } from "three";

import { SceneMap } from "../types";

export function removeColliderVisual(nodeId: string, map: SceneMap) {
  const collider = map.colliders.get(nodeId);
  if (!collider) return;

  map.colliders.delete(nodeId);
  collider.removeFromParent();
  collider.traverse((child) => {
    if (child instanceof Mesh) child.geometry.dispose();
  });
}
