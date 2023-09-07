import { EditNode_Collider_Type } from "@unavi/protocol";
import {
  BoxCollider,
  CapsuleCollider,
  CylinderCollider,
  HullCollider,
  MeshCollider,
  SphereCollider,
} from "lattice-engine/physics";
import { Name } from "lattice-engine/scene";
import { Commands, Entity, EventReader, Mut, Query } from "thyseus";

import { EditCollider } from "../events";

export function editColliders(
  commands: Commands,
  events: EventReader<EditCollider>,
  names: Query<[Entity, Name]>,
  boxColliders: Query<[Entity, Mut<BoxCollider>]>,
  sphereColliders: Query<[Entity, Mut<SphereCollider>]>,
  capsuleColliders: Query<[Entity, Mut<CapsuleCollider>]>,
  cylinderColliders: Query<[Entity, Mut<CylinderCollider>]>,
  meshColliders: Query<[Entity, Mut<MeshCollider>]>,
  hullColliders: Query<[Entity, Mut<HullCollider>]>
) {
  if (events.length === 0) return;

  for (const e of events) {
    for (const [entity, name] of names) {
      if (name.value !== e.target) continue;

      let foundCollider = false;

      // Remove or update collider
      for (const [colliderEntity, collider] of boxColliders) {
        if (colliderEntity.id !== entity.id) continue;

        collider.size.fromArray(e.size);

        if (e.type === EditNode_Collider_Type.BOX) {
          foundCollider = true;
        } else {
          commands.get(colliderEntity).remove(BoxCollider);
        }

        break;
      }

      for (const [colliderEntity, collider] of sphereColliders) {
        if (colliderEntity.id !== entity.id) continue;

        collider.radius = e.radius;

        if (e.type === EditNode_Collider_Type.SPHERE) {
          foundCollider = true;
        } else {
          commands.get(colliderEntity).remove(SphereCollider);
        }

        break;
      }

      for (const [colliderEntity, collider] of capsuleColliders) {
        if (colliderEntity.id !== entity.id) continue;

        collider.radius = e.radius;
        collider.height = e.height;

        if (e.type === EditNode_Collider_Type.CAPSULE) {
          foundCollider = true;
        } else {
          commands.get(colliderEntity).remove(CapsuleCollider);
        }

        break;
      }

      for (const [colliderEntity, collider] of cylinderColliders) {
        if (colliderEntity.id !== entity.id) continue;

        collider.radius = e.radius;
        collider.height = e.height;

        if (e.type === EditNode_Collider_Type.CYLINDER) {
          foundCollider = true;
        } else {
          commands.get(colliderEntity).remove(CylinderCollider);
        }

        break;
      }

      for (const [colliderEntity, collider] of meshColliders) {
        if (colliderEntity.id !== entity.id) continue;

        if (e.type === EditNode_Collider_Type.MESH) {
          foundCollider = true;
        } else {
          commands.get(colliderEntity).remove(MeshCollider);
        }

        break;
      }

      for (const [colliderEntity, collider] of hullColliders) {
        if (colliderEntity.id !== entity.id) continue;

        if (e.type === EditNode_Collider_Type.HULL) {
          foundCollider = true;
        } else {
          commands.get(colliderEntity).remove(HullCollider);
        }

        break;
      }

      if (foundCollider || e.type === EditNode_Collider_Type.NONE) {
        break;
      }

      // Create new collider
      if (e.type === EditNode_Collider_Type.BOX) {
        const collider = new BoxCollider();
        collider.size.fromArray(e.size);

        commands.get(entity).add(collider);
      }

      if (e.type === EditNode_Collider_Type.SPHERE) {
        const collider = new SphereCollider();
        collider.radius = e.radius;

        commands.get(entity).add(collider);
      }

      if (e.type === EditNode_Collider_Type.CYLINDER) {
        const collider = new CylinderCollider();
        collider.radius = e.radius;
        collider.height = e.height;

        commands.get(entity).add(collider);
      }

      if (e.type === EditNode_Collider_Type.CAPSULE) {
        const collider = new CapsuleCollider();
        collider.radius = e.radius;
        collider.height = e.height;

        commands.get(entity).add(collider);
      }

      if (e.type === EditNode_Collider_Type.MESH) {
        const collider = new MeshCollider();
        // TODO: set mesh

        commands.get(entity).add(collider);
      }

      if (e.type === EditNode_Collider_Type.HULL) {
        const collider = new HullCollider();
        // TODO: set mesh

        commands.get(entity).add(collider);
      }
    }
  }

  events.clear();
}
