import { Mesh, Name, Parent, Transform } from "lattice-engine/scene";
import { Entity, EventReader, Mut, Query } from "thyseus";

import { EditNode } from "../events";

export function editNodes(
  events: EventReader<EditNode>,
  named: Query<[Entity, Name]>,
  nodes: Query<[Entity, Mut<Name>, Mut<Parent>, Mut<Transform>]>,
  meshes: Query<[Name, Mut<Mesh>]>
) {
  if (events.length === 0) return;

  for (const e of events) {
    for (const [entity, name, parent, transform] of nodes) {
      if (name.value !== e.target) continue;

      if (e.name) {
        name.value = e.name;
      }

      if (e.parentName) {
        parent.id = 0n;

        for (const [entity, name] of named) {
          if (name.value === e.parentName) {
            parent.id = entity.id;
            break;
          }
        }
      }

      if (e.meshName) {
        for (const [name, mesh] of meshes) {
          if (name.value === e.meshName) {
            mesh.parentId = entity.id;
          }
        }
      }

      if (e.translation) {
        transform.translation.copy(e.transform.translation);
      }

      if (e.rotation) {
        transform.rotation.copy(e.transform.rotation);
      }

      if (e.scale) {
        transform.scale.copy(e.transform.scale);
      }
    }
  }

  events.clear();
}
