import { ColliderType } from "@unavi/protocol";
import {
  BoxCollider,
  CapsuleCollider,
  CylinderCollider,
  HullCollider,
  MeshCollider,
  SphereCollider,
} from "lattice-engine/physics";
import { Name } from "lattice-engine/scene";
import { Commands, dropStruct, Entity, EventReader, Mut, Query } from "thyseus";

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
  hullColliders: Query<[Entity, Mut<HullCollider>]>,
) {
  if (events.length === 0) return;

  for (const e of events) {
    for (const [entity, name] of names) {
      if (name.value !== e.target) continue;

      let foundCollider = false;

      // Remove or update collider
      for (const [colliderEntity, collider] of boxColliders) {
        if (colliderEntity.id !== entity.id) continue;

        collider.size.array.set(e.size);

        if (e.type === ColliderType.Box) {
          foundCollider = true;
        } else {
          commands.get(colliderEntity).remove(BoxCollider);
        }

        break;
      }

      for (const [colliderEntity, collider] of sphereColliders) {
        if (colliderEntity.id !== entity.id) continue;

        collider.radius = e.radius;

        if (e.type === ColliderType.Sphere) {
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

        if (e.type === ColliderType.Capsule) {
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

        if (e.type === ColliderType.Cylinder) {
          foundCollider = true;
        } else {
          commands.get(colliderEntity).remove(CylinderCollider);
        }

        break;
      }

      for (const [colliderEntity, collider] of meshColliders) {
        if (colliderEntity.id !== entity.id) continue;

        if (e.type === ColliderType.Mesh) {
          foundCollider = true;
        } else {
          commands.get(colliderEntity).remove(MeshCollider);
        }

        break;
      }

      for (const [colliderEntity, collider] of hullColliders) {
        if (colliderEntity.id !== entity.id) continue;

        if (e.type === ColliderType.Hull) {
          foundCollider = true;
        } else {
          commands.get(colliderEntity).remove(HullCollider);
        }

        break;
      }

      if (foundCollider || e.type === "none") {
        break;
      }

      // Create new collider
      if (e.type === ColliderType.Box) {
        const collider = new BoxCollider();
        collider.size.array.set(e.size);

        commands.get(entity).add(collider);

        dropStruct(collider);
      }

      if (e.type === ColliderType.Sphere) {
        const collider = new SphereCollider();
        collider.radius = e.radius;

        commands.get(entity).add(collider);

        dropStruct(collider);
      }

      if (e.type === ColliderType.Cylinder) {
        const collider = new CylinderCollider();
        collider.radius = e.radius;
        collider.height = e.height;

        commands.get(entity).add(collider);

        dropStruct(collider);
      }

      if (e.type === ColliderType.Capsule) {
        const collider = new CapsuleCollider();
        collider.radius = e.radius;
        collider.height = e.height;

        commands.get(entity).add(collider);

        dropStruct(collider);
      }

      if (e.type === ColliderType.Mesh) {
        const collider = new MeshCollider();
        // TODO: set mesh

        commands.get(entity).add(collider);

        dropStruct(collider);
      }

      if (e.type === ColliderType.Hull) {
        const collider = new HullCollider();
        // TODO: set mesh

        commands.get(entity).add(collider);

        dropStruct(collider);
      }
    }
  }

  events.clear();
}
