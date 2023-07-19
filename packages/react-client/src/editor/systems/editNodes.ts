import { Mesh, Name, Parent } from "lattice-engine/scene";
import { Entity, EventReader, Mut, Query } from "thyseus";

import { EditNode } from "../events";

export function editNodes(
  events: EventReader<EditNode>,
  named: Query<[Entity, Name]>,
  nodes: Query<[Entity, Mut<Name>, Mut<Parent>]>,
  meshes: Query<[Name, Mut<Mesh>]>
) {
  if (events.length === 0) return;

  for (const e of events) {
    for (const [entity, name, parent] of nodes) {
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
    }
  }

  events.clear();
}
