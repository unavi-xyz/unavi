import { SubScene } from "houseki/gltf";
import { Name } from "houseki/scene";
import { Commands, Entity, Mut, Query } from "thyseus";

import { EditorId } from "../../client/components";
import { syncedStore } from "../store";

export function createScenes(
  commands: Commands,
  scenes: Query<[Entity, EditorId, Mut<Name>, Mut<SubScene>]>,
  ents: Query<[Entity, EditorId]>
) {
  const ids: string[] = [];

  for (const [entity, id, name, subscene] of scenes) {
    ids.push(id.value);

    const scene = syncedStore.scenes[id.value];

    if (!scene) {
      // Scene was deleted, remove from ecs
      commands.get(entity).despawn();
    } else {
      // Sync data
      name.value = scene.name;

      subscene.nodes.length = 0;

      for (const nodeId of scene.nodeIds) {
        for (const [ent, entId] of ents) {
          if (nodeId === entId.value) {
            subscene.nodes.push(ent.id);
          }
        }
      }
    }
  }

  // Create new scenes
  for (const id of Object.keys(syncedStore.scenes)) {
    if (ids.includes(id)) continue;

    commands.spawn(true).add(new EditorId(id)).addType(Name).addType(SubScene);
  }
}
