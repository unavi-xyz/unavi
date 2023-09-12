import { Mesh, Name, Parent, Transform } from "houseki/scene";
import { Entity, EventReader, Mut, Query } from "thyseus";

import { EditorId } from "../../client/components";
import { EditNode } from "../events";

export function editNodes(
  events: EventReader<EditNode>,
  ids: Query<[Entity, EditorId]>,
  nodes: Query<[Entity, EditorId, Mut<Name>, Mut<Parent>, Mut<Transform>]>,
  meshes: Query<[EditorId, Mut<Mesh>]>
) {
  if (events.length === 0) return;

  for (const e of events) {
    for (const [entity, id, name, parent, transform] of nodes) {
      if (id.value !== e.target) continue;

      if (e.name) {
        name.value = e.name;
      }

      if (e.parent) {
        parent.id = 0n;

        for (const [parentEnt, parentId] of ids) {
          if (parentId.value === e.parent) {
            parent.id = parentEnt.id;
            break;
          }
        }
      }

      if (e.mesh) {
        for (const [meshId, mesh] of meshes) {
          if (meshId.value === e.mesh) {
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
