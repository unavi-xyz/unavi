import { Extra } from "houseki/gltf";
import { Commands, Entity, EventReader, Mut, Query, SystemRes } from "thyseus";

import { EditorId } from "../../client/components";
import { EditExtra } from "../events";

class LocalStore {
  /**
   * Entity ID -> ID
   */
  targets = new Map<bigint, string>();
}

export function editExtras(
  commands: Commands,
  localStore: SystemRes<LocalStore>,
  events: EventReader<EditExtra>,
  extras: Query<[Entity, Mut<Extra>]>,
  ids: Query<[Entity, EditorId]>
) {
  if (events.length === 0) return;

  for (const [entity, id] of ids) {
    localStore.targets.set(entity.id, id.value);
  }

  for (const event of events) {
    let foundExtra = false;

    // Find existing extra to update
    for (const [entity, extra] of extras) {
      const extraId = localStore.targets.get(extra.target);
      if (extraId !== event.target) continue;

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

      for (const [id, name] of localStore.targets) {
        if (name === event.target) {
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
  localStore.targets.clear();
}
