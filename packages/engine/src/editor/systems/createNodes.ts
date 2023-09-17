import {
  BoxCollider,
  CapsuleCollider,
  CylinderCollider,
  HullCollider,
  MeshCollider,
  SphereCollider,
} from "houseki/physics";
import { GlobalTransform, Name, Parent, Transform } from "houseki/scene";
import { Commands, Entity, Mut, Query } from "thyseus";

import { EditorId } from "../../client/components";
import { getEntityId } from "../entities";
import { SyncedNode_Collider_Type, syncedStore } from "../store";

export function createNodes(
  commands: Commands,
  nodes: Query<[Entity, EditorId, Mut<Name>, Mut<Transform>, Mut<Parent>]>,
  boxColliders: Query<[Entity, Mut<BoxCollider>]>,
  sphereColliders: Query<[Entity, Mut<SphereCollider>]>,
  capsuleColliders: Query<[Entity, Mut<CapsuleCollider>]>,
  cylinderColliders: Query<[Entity, Mut<CylinderCollider>]>,
  meshColliders: Query<[Entity, Mut<MeshCollider>]>,
  hullColliders: Query<[Entity, Mut<HullCollider>]>
) {
  const ids: string[] = [];

  for (const [entity, id, name, transform, parent] of nodes) {
    ids.push(id.value);

    const node = syncedStore.nodes[id.value];

    if (!node) {
      // Node was deleted, remove from ecs
      commands.get(entity).despawn();
    } else {
      // Sync data
      name.value = node.name;

      transform.translation.fromArray(node.translation);
      transform.rotation.fromArray(node.rotation);
      transform.scale.fromArray(node.scale);

      if (node.parentId) {
        parent.id = getEntityId(node.parentId) ?? 0n;
      } else {
        parent.id = 0n;
      }

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
            commands.get(entity).add(new BoxCollider(node.collider.size));
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
            commands.get(entity).add(new SphereCollider(node.collider.radius));
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
              .add(
                new CapsuleCollider(node.collider.radius, node.collider.height)
              );
          }
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
            commands.get(entity).add(collider);
          }

          break;
        }

        case SyncedNode_Collider_Type.CONVEX: {
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
            commands.get(entity).add(collider);
          }

          break;
        }
      }
    }
  }

  // Create new nodes
  for (const id of Object.keys(syncedStore.nodes)) {
    if (!ids.includes(id)) {
      commands
        .spawn(true)
        .add(new EditorId(id))
        .addType(Name)
        .addType(Transform)
        .addType(GlobalTransform)
        .addType(Parent);
    }
  }
}
