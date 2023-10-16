import { Name, Scene } from "houseki/scene";
import { Commands, Entity, Mut, Query, With } from "thyseus";

import { EditorId } from "../../client/components";
import { syncedStore } from "../store";

export function createScenes(
  commands: Commands,
  scenes: Query<[Entity, EditorId, Mut<Name>], With<Scene>>
) {
  const ids: string[] = [];

  for (const [entity, id, name] of scenes) {
    ids.push(id.value);

    const scene = syncedStore.scenes[id.value];

    if (!scene) {
      // Scene was deleted, remove from ecs
      commands.get(entity).despawn();
    } else {
      // Sync data
      name.value = scene.name;
    }
  }

  // Create new scenes
  for (const id of Object.keys(syncedStore.scenes)) {
    if (ids.includes(id)) continue;
    commands.spawn(true).add(new EditorId(id)).addType(Name).addType(Scene);
  }
}
