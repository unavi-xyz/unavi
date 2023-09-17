import { SubScene } from "houseki/gltf";
import {
  BoxCollider,
  CapsuleCollider,
  CylinderCollider,
  HullCollider,
  MeshCollider,
  SphereCollider,
} from "houseki/physics";
import { Parent, Transform } from "houseki/scene";
import { Commands, Entity, Mut, Query, With, Without } from "thyseus";

import { EditorId } from "../../client/components";
import { getEntityId } from "../entities";
import { SyncedNode_Collider_Type, syncedStore } from "../store";

export function createColliders(
  commands: Commands,
  nodes: Query<
    [Entity, EditorId],
    [With<Transform>, With<Parent>, Without<SubScene>]
  >,
  boxColliders: Query<[Entity, Mut<BoxCollider>]>,
  sphereColliders: Query<[Entity, Mut<SphereCollider>]>,
  capsuleColliders: Query<[Entity, Mut<CapsuleCollider>]>,
  cylinderColliders: Query<[Entity, Mut<CylinderCollider>]>,
  meshColliders: Query<[Entity, Mut<MeshCollider>]>,
  hullColliders: Query<[Entity, Mut<HullCollider>]>
) {
  for (const [entity, id] of nodes) {
    const node = syncedStore.nodes[id.value];
    if (!node) continue;

    let foundBox = false;
    let foundSphere = false;
    let foundCapsule = false;
    let foundCylinder = false;
    let foundMesh = false;
    let foundHull = false;

    for (const [ent, collider] of boxColliders) {
      if (ent.id !== entity.id) continue;

      if (node.collider.type !== SyncedNode_Collider_Type.BOX) {
        commands.get(entity).remove(BoxCollider);
        break;
      }

      foundBox = true;

      collider.size.fromArray(node.collider.size);
    }

    if (!foundBox && node.collider.type === SyncedNode_Collider_Type.BOX) {
      commands.get(entity).add(new BoxCollider(node.collider.size));
    }

    for (const [ent, collider] of sphereColliders) {
      if (ent.id !== entity.id) continue;

      if (node.collider.type !== SyncedNode_Collider_Type.SPHERE) {
        commands.get(entity).remove(SphereCollider);
        break;
      }

      foundSphere = true;

      collider.radius = node.collider.radius;
    }

    if (
      !foundSphere &&
      node.collider.type === SyncedNode_Collider_Type.SPHERE
    ) {
      commands.get(entity).add(new SphereCollider(node.collider.radius));
    }

    for (const [ent, collider] of capsuleColliders) {
      if (ent.id !== entity.id) continue;

      if (node.collider.type !== SyncedNode_Collider_Type.CAPSULE) {
        commands.get(entity).remove(CapsuleCollider);
        break;
      }

      foundCapsule = true;

      collider.radius = node.collider.radius;
      collider.height = node.collider.height;
    }

    if (
      !foundCapsule &&
      node.collider.type === SyncedNode_Collider_Type.CAPSULE
    ) {
      commands
        .get(entity)
        .add(new CapsuleCollider(node.collider.radius, node.collider.height));
    }

    for (const [ent, collider] of cylinderColliders) {
      if (ent.id !== entity.id) continue;

      if (node.collider.type !== SyncedNode_Collider_Type.CYLINDER) {
        commands.get(entity).remove(CylinderCollider);
        break;
      }

      foundCylinder = true;

      collider.radius = node.collider.radius;
      collider.height = node.collider.height;
    }

    if (
      !foundCylinder &&
      node.collider.type === SyncedNode_Collider_Type.CYLINDER
    ) {
      commands
        .get(entity)
        .add(new CylinderCollider(node.collider.radius, node.collider.height));
    }

    for (const [ent, collider] of meshColliders) {
      if (ent.id !== entity.id) continue;

      if (node.collider.type !== SyncedNode_Collider_Type.MESH) {
        commands.get(entity).remove(MeshCollider);
        break;
      }

      foundMesh = true;

      const meshId = getEntityId(node.collider.meshId);
      if (meshId) {
        collider.meshId = meshId;
      } else {
        collider.meshId = entity.id;
      }
    }

    if (!foundMesh && node.collider.type === SyncedNode_Collider_Type.MESH) {
      const mesh = new MeshCollider();

      const meshId = getEntityId(node.collider.meshId);
      if (meshId) {
        mesh.meshId = meshId;
      } else {
        mesh.meshId = entity.id;
      }

      commands.get(entity).add(mesh);
    }

    for (const [ent, collider] of hullColliders) {
      if (ent.id !== entity.id) continue;

      if (node.collider.type !== SyncedNode_Collider_Type.HULL) {
        commands.get(entity).remove(HullCollider);
        break;
      }

      foundHull = true;

      const meshId = getEntityId(node.collider.meshId);
      if (meshId) {
        collider.meshId = meshId;
      } else {
        collider.meshId = entity.id;
      }
    }

    if (!foundHull && node.collider.type === SyncedNode_Collider_Type.HULL) {
      const mesh = new HullCollider();

      const meshId = getEntityId(node.collider.meshId);
      if (meshId) {
        mesh.meshId = meshId;
      } else {
        mesh.meshId = entity.id;
      }

      commands.get(entity).add(mesh);
    }
  }
}
