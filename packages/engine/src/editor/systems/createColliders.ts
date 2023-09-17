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

    switch (node.collider.type) {
      case SyncedNode_Collider_Type.BOX: {
        let found = false;

        for (const [ent, collider] of boxColliders) {
          if (ent.id === entity.id) {
            found = true;
            collider.size.fromArray(node.collider.size);
          }
        }

        if (!found) {
          commands
            .get(entity)
            .remove(SphereCollider)
            .remove(CapsuleCollider)
            .remove(CylinderCollider)
            .remove(MeshCollider)
            .remove(HullCollider)
            .add(new BoxCollider(node.collider.size));
        }

        break;
      }

      case SyncedNode_Collider_Type.SPHERE: {
        let found = false;

        for (const [ent, collider] of sphereColliders) {
          if (ent.id === entity.id) {
            found = true;
            collider.radius = node.collider.radius;
          }
        }

        if (!found) {
          commands
            .get(entity)
            .remove(BoxCollider)
            .remove(CapsuleCollider)
            .remove(CylinderCollider)
            .remove(MeshCollider)
            .remove(HullCollider)
            .add(new SphereCollider(node.collider.radius));
        }

        break;
      }

      case SyncedNode_Collider_Type.CAPSULE: {
        let found = false;

        for (const [ent, collider] of capsuleColliders) {
          if (ent.id === entity.id) {
            found = true;
            collider.radius = node.collider.radius;
            collider.height = node.collider.height;
          }
        }

        if (!found) {
          commands
            .get(entity)
            .remove(BoxCollider)
            .remove(SphereCollider)
            .remove(CylinderCollider)
            .remove(MeshCollider)
            .remove(HullCollider)
            .add(
              new CapsuleCollider(node.collider.radius, node.collider.height)
            );
        }

        break;
      }

      case SyncedNode_Collider_Type.CYLINDER: {
        let found = false;

        for (const [ent, collider] of cylinderColliders) {
          if (ent.id === entity.id) {
            found = true;
            collider.radius = node.collider.radius;
            collider.height = node.collider.height;
          }
        }

        if (!found) {
          commands
            .get(entity)
            .remove(BoxCollider)
            .remove(SphereCollider)
            .remove(CapsuleCollider)
            .remove(MeshCollider)
            .remove(HullCollider)
            .add(
              new CylinderCollider(node.collider.radius, node.collider.height)
            );
        }

        break;
      }

      case SyncedNode_Collider_Type.MESH: {
        let found = false;

        const meshId = getEntityId(node.collider.meshId);
        if (!meshId) break;

        for (const [ent, collider] of meshColliders) {
          if (ent.id === entity.id) {
            found = true;
            collider.meshId = meshId;
          }
        }

        if (!found) {
          const collider = new MeshCollider();
          collider.meshId = meshId;
          commands
            .get(entity)
            .remove(BoxCollider)
            .remove(SphereCollider)
            .remove(CapsuleCollider)
            .remove(CylinderCollider)
            .remove(HullCollider)
            .add(collider);
        }

        break;
      }

      case SyncedNode_Collider_Type.HULL: {
        let found = false;

        const meshId = getEntityId(node.collider.meshId);
        if (!meshId) break;

        for (const [ent, collider] of hullColliders) {
          if (ent.id === entity.id) {
            found = true;
            collider.meshId = meshId;
          }
        }

        if (!found) {
          const collider = new HullCollider();
          collider.meshId = meshId;
          commands
            .get(entity)
            .remove(BoxCollider)
            .remove(SphereCollider)
            .remove(CapsuleCollider)
            .remove(CylinderCollider)
            .remove(MeshCollider)
            .add(collider);
        }

        break;
      }
    }
  }
}
