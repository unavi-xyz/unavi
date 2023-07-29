import { DynamicBody, StaticBody } from "lattice-engine/physics";
import { Name } from "lattice-engine/scene";
import { Commands, Entity, EventReader, Query } from "thyseus";

import { EditRigidBody } from "../events";

export function editRigidBodies(
  commands: Commands,
  events: EventReader<EditRigidBody>,
  names: Query<[Entity, Name]>,
) {
  if (events.length === 0) return;

  for (const e of events) {
    for (const [entity, name] of names) {
      if (name.value !== e.target) continue;

      if (e.type === "none") {
        commands.get(entity).remove(DynamicBody).remove(StaticBody);
        break;
      }

      if (e.type === "static") {
        commands.get(entity).remove(DynamicBody).addType(StaticBody);
        break;
      }

      if (e.type === "dynamic") {
        commands.get(entity).remove(StaticBody).addType(DynamicBody);
        break;
      }
    }
  }

  events.clear();
}
