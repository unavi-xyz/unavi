import { Warehouse } from "lattice-engine/core";
import { Extra } from "lattice-engine/gltf";
import { Name } from "lattice-engine/scene";
import { Commands, Entity, EventReader, Mut, Query, Res } from "thyseus";

import { EditExtra } from "../events";

/**
 * Entity ID -> Name
 */
const nameMap = new Map<bigint, string>();

export function editExtras(
  commands: Commands,
  warehouse: Res<Mut<Warehouse>>,
  events: EventReader<EditExtra>,
  extras: Query<[Entity, Mut<Extra>]>,
  names: Query<[Entity, Name]>
) {
  if (events.length === 0) return;

  for (const [entity, name] of names) {
    const value = name.value.read(warehouse) ?? "";
    nameMap.set(entity.id, value);
  }

  for (const event of events) {
    let foundExtra = false;

    const eventTarget = event.target.read(warehouse) ?? "";

    // Find existing extra to update
    for (const [entity, extra] of extras) {
      const extraName = nameMap.get(extra.target);
      if (extraName !== eventTarget) continue;

      const eventKey = event.key.read(warehouse) ?? "";
      const extraKey = extra.key.read(warehouse) ?? "";
      if (eventKey !== extraKey) continue;

      const eventValue = event.value.read(warehouse) ?? "";

      foundExtra = true;

      if (eventValue) {
        extra.value.write(eventValue, warehouse);
      } else {
        // Delete extra if empty value
        commands.despawn(entity);
      }
    }

    // Create new extra if not found
    if (!foundExtra) {
      const extra = new Extra();

      for (const [id, name] of nameMap) {
        if (name === eventTarget) {
          extra.target = id;
          break;
        }
      }

      extra.key = event.key;
      extra.value = event.value;

      commands.spawn(true).add(extra);
    }
  }

  events.clear();
  nameMap.clear();
}
