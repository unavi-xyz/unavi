import { Extra } from "lattice-engine/gltf";
import { Name } from "lattice-engine/scene";
import { Commands, dropStruct, Entity, EventReader, Mut, Query } from "thyseus";

import { EditExtra } from "../events";

/**
 * Entity ID -> Name
 */
const nameMap = new Map<bigint, string>();

export function editExtras(
  commands: Commands,
  events: EventReader<EditExtra>,
  extras: Query<[Entity, Mut<Extra>]>,
  names: Query<[Entity, Name]>
) {
  if (events.length === 0) return;

  for (const [entity, name] of names) {
    nameMap.set(entity.id, name.value);
  }

  for (const event of events) {
    let foundExtra = false;

    // Find existing extra to update
    for (const [entity, extra] of extras) {
      const extraName = nameMap.get(extra.target);
      if (extraName !== event.target) continue;
      if (event.key !== extra.key) continue;

      foundExtra = true;

      if (event.value) {
        extra.value = event.value;
      } else {
        // Delete extra if empty value
        commands.despawn(entity);
      }
    }

    // Create new extra if not found
    if (!foundExtra) {
      const extra = new Extra();

      for (const [id, name] of nameMap) {
        if (name === event.target) {
          extra.target = id;
          break;
        }
      }

      extra.key = event.key;
      extra.value = event.value;

      commands.spawn(true).add(extra);

      dropStruct(extra);
    }
  }

  events.clear();
  nameMap.clear();
}
