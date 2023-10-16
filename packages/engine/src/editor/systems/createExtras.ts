import { Extra } from "houseki/gltf";
import { Parent, Transform } from "houseki/scene";
import { Commands, Entity, Mut, Query, With } from "thyseus";

import { EditorId } from "../../client/components";
import { syncedStore } from "../store";

export function createExtras(
  commands: Commands,
  nodes: Query<[Entity, EditorId], [With<Transform>, With<Parent>]>,
  extras: Query<Mut<Extra>>
) {
  for (const [entity, id] of nodes) {
    const node = syncedStore.nodes[id.value];
    if (!node) continue;

    for (const [key, value] of Object.entries(node.extras)) {
      let found = false;

      for (const extra of extras) {
        if (extra.target !== entity.id) continue;
        if (extra.key !== key) continue;

        found = true;
        extra.value = JSON.stringify(value);
      }

      if (!found) {
        const extra = new Extra(key, JSON.stringify(value));
        extra.target = entity.id;
        commands.spawn(true).add(extra);
      }
    }
  }
}
