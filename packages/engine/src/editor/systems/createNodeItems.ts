import { EditNode_Collider_Type } from "@unavi/protocol";
import { Extra } from "houseki/gltf";
import {
  BoxCollider,
  CapsuleCollider,
  CylinderCollider,
  HullCollider,
  MeshCollider,
  SphereCollider,
} from "houseki/physics";
import { Name, Parent, Scene, SceneStruct, Transform } from "houseki/scene";
import { Entity, Query, Res } from "thyseus";

import { EditorId } from "../../client/components";
import { NodeItem } from "../classes/NodeItem";
import { useSceneStore } from "../sceneStore";

export function createNodeItems(
  sceneStruct: Res<SceneStruct>,
  nodes: Query<[Entity, Transform, Parent, EditorId]>,
  scenes: Query<[Entity, Scene]>,
  names: Query<[Entity, Name]>,
  extras: Query<Extra>,
  boxColliders: Query<[Entity, BoxCollider]>,
  sphereColliders: Query<[Entity, SphereCollider]>,
  cylinderColliders: Query<[Entity, CylinderCollider]>,
  capsuleColliders: Query<[Entity, CapsuleCollider]>,
  meshColliders: Query<[Entity, MeshCollider]>,
  hullColliders: Query<[Entity, HullCollider]>
) {
  const { enabled } = useSceneStore.getState();
  if (!enabled) return;

  let items = useSceneStore.getState().items;

  // Set root id
  for (const [entity, scene] of scenes) {
    if (sceneStruct.activeScene === entity.id) {
      useSceneStore.setState({ rootId: scene.rootId });
    }
  }

  const ids: bigint[] = [];

  // Create or update items
  for (const [entity, transform, parent, id] of nodes) {
    ids.push(entity.id);

    let item = items.get(entity.id);

    if (!item) {
      item = new NodeItem(id.value, entity.id);

      items = new Map(items);
      items.set(entity.id, item);

      useSceneStore.setState({ items });
    }

    item.translation[0] = transform.translation.x;
    item.translation[1] = transform.translation.y;
    item.translation[2] = transform.translation.z;

    item.rotation[0] = transform.rotation.x;
    item.rotation[1] = transform.rotation.y;
    item.rotation[2] = transform.rotation.z;
    item.rotation[3] = transform.rotation.w;

    item.scale[0] = transform.scale.x;
    item.scale[1] = transform.scale.y;
    item.scale[2] = transform.scale.z;

    if (parent) {
      item.parentId = parent.id;
    } else {
      item.parentId = undefined;
    }
  }

  for (const [entity, name] of names) {
    const item = items.get(entity.id);
    if (!item) continue;

    item.name = name.value;
  }

  for (const extra of extras) {
    try {
      const item = items.get(extra.target);
      if (!item) continue;

      if (extra.key === "locked") {
        item.locked = JSON.parse(extra.value);
      }
    } catch {
      // Ignore invalid JSON
    }
  }

  for (const [entity, boxCollider] of boxColliders) {
    const item = items.get(entity.id);
    if (!item) continue;

    item.colliderType = EditNode_Collider_Type.BOX;
    item.collider.size[0] = boxCollider.size.x;
    item.collider.size[1] = boxCollider.size.y;
    item.collider.size[2] = boxCollider.size.z;
  }

  for (const [entity, sphereCollider] of sphereColliders) {
    const item = items.get(entity.id);
    if (!item) continue;

    item.colliderType = EditNode_Collider_Type.SPHERE;
    item.collider.radius = sphereCollider.radius;
  }

  for (const [entity, cylinderCollider] of cylinderColliders) {
    const item = items.get(entity.id);
    if (!item) continue;

    item.colliderType = EditNode_Collider_Type.CYLINDER;
    item.collider.radius = cylinderCollider.radius;
    item.collider.height = cylinderCollider.height;
  }

  for (const [entity, capsuleCollider] of capsuleColliders) {
    const item = items.get(entity.id);
    if (!item) continue;

    item.colliderType = EditNode_Collider_Type.CAPSULE;
    item.collider.radius = capsuleCollider.radius;
    item.collider.height = capsuleCollider.height;
  }

  for (const [entity, meshCollider] of meshColliders) {
    const item = items.get(entity.id);
    if (!item) continue;

    item.colliderType = EditNode_Collider_Type.MESH;
    item.collider.meshId = meshCollider.meshId;
  }

  for (const [entity, hullCollider] of hullColliders) {
    const item = items.get(entity.id);
    if (!item) continue;

    item.colliderType = EditNode_Collider_Type.HULL;
    item.collider.meshId = hullCollider.meshId;
  }

  // Destroy items that no longer exist
  for (const [id, item] of items.entries()) {
    if (!ids.includes(id)) {
      item.destroy();
    }
  }
}
