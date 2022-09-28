import { SceneMap } from "./SceneMap";

export function removeColliderVisual(entityId: string, map: SceneMap) {
  const collider = map.colliders.get(entityId);
  if (!collider) return;

  map.colliders.delete(entityId);
  collider.removeFromParent();
  collider.geometry.dispose();
}
