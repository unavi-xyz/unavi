import {
  BoxCollider,
  CapsuleCollider,
  CylinderCollider,
  DynamicBody,
  HullCollider,
  MeshCollider,
  SphereCollider,
  StaticBody,
} from "lattice-engine/physics";
import { Entity, Query, With } from "thyseus";

import { useSceneStore } from "../sceneStore";
import { ColliderType, RigidBodyType } from "../types";

export function createTreeItemsPhysics(
  boxColliders: Query<[Entity, BoxCollider]>,
  sphereColliders: Query<[Entity, SphereCollider]>,
  capsuleColliders: Query<[Entity, CapsuleCollider]>,
  cylinderColliders: Query<[Entity, CylinderCollider]>,
  meshColliders: Query<[Entity, MeshCollider]>,
  hullColliders: Query<[Entity, HullCollider]>,
  staticBodies: Query<Entity, With<StaticBody>>,
  dynamicBodies: Query<Entity, With<DynamicBody>>
) {
  const { enabled, items } = useSceneStore.getState();
  if (!enabled) return;

  const colliderIds: bigint[] = [];

  for (const [entity, boxCollider] of boxColliders) {
    const item = items.get(entity.id);
    if (!item) continue;

    colliderIds.push(entity.id);

    item.colliderType = ColliderType.Box;
    item.collider.size[0] = boxCollider.size.x;
    item.collider.size[1] = boxCollider.size.y;
    item.collider.size[2] = boxCollider.size.z;
  }

  for (const [entity, sphereCollider] of sphereColliders) {
    const item = items.get(entity.id);
    if (!item) continue;

    colliderIds.push(entity.id);

    item.colliderType = ColliderType.Sphere;
    item.collider.radius = sphereCollider.radius;
  }

  for (const [entity, capsuleCollider] of capsuleColliders) {
    const item = items.get(entity.id);
    if (!item) continue;

    colliderIds.push(entity.id);

    item.colliderType = ColliderType.Capsule;
    item.collider.radius = capsuleCollider.radius;
    item.collider.height = capsuleCollider.height;
  }

  for (const [entity, cylinderCollider] of cylinderColliders) {
    const item = items.get(entity.id);
    if (!item) continue;

    colliderIds.push(entity.id);

    item.colliderType = ColliderType.Cylinder;
    item.collider.radius = cylinderCollider.radius;
    item.collider.height = cylinderCollider.height;
  }

  for (const [entity, meshCollider] of meshColliders) {
    const item = items.get(entity.id);
    if (!item) continue;

    colliderIds.push(entity.id);

    item.colliderType = ColliderType.Mesh;
    item.collider.meshId = meshCollider.meshId;
  }

  for (const [entity, hullCollider] of hullColliders) {
    const item = items.get(entity.id);
    if (!item) continue;

    colliderIds.push(entity.id);

    item.colliderType = ColliderType.Hull;
    item.collider.meshId = hullCollider.meshId;
  }

  const bodyIds: bigint[] = [];

  for (const entity of staticBodies) {
    const item = items.get(entity.id);
    if (!item) continue;

    bodyIds.push(entity.id);

    item.rigidBodyType = RigidBodyType.Static;
  }

  for (const entity of dynamicBodies) {
    const item = items.get(entity.id);
    if (!item) continue;

    bodyIds.push(entity.id);

    item.rigidBodyType = RigidBodyType.Dynamic;
  }

  // Remove old data
  for (const [id, item] of items) {
    if (!colliderIds.includes(id)) {
      item.colliderType = undefined;
    }

    if (!bodyIds.includes(id)) {
      item.rigidBodyType = undefined;
    }
  }
}