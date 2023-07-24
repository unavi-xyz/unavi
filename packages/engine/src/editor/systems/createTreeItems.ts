import { Extra } from "lattice-engine/gltf";
import {
  Name,
  Parent,
  Scene,
  SceneStruct,
  Transform,
} from "lattice-engine/scene";
import { Entity, Query, Res } from "thyseus";

import { TreeItem } from "../classes/TreeItem";
import { useSceneStore } from "../sceneStore";

/**
 * Updates the scene tree with the latest entities.
 */
export function createTreeItems(
  nodes: Query<[Entity, Transform, Parent]>,
  scenes: Query<[Entity, Scene]>,
  names: Query<[Entity, Name]>,
  extras: Query<Extra>,
  sceneStruct: Res<SceneStruct>
) {
  const { enabled, items } = useSceneStore.getState();
  if (!enabled) return;

  // Set root id
  for (const [entity, scene] of scenes) {
    if (sceneStruct.activeScene === entity.id) {
      useSceneStore.setState({ rootId: scene.rootId });
    }
  }

  const ids: bigint[] = [];

  // Create or update items
  for (const [entity, transform, parent] of nodes) {
    ids.push(entity.id);

    let item = items.get(entity.id);

    if (!item) {
      item = new TreeItem(entity.id);
      items.set(entity.id, item);
    }

    item.translation[0] = transform.translation.x;
    item.translation[1] = transform.translation.y;
    item.translation[2] = transform.translation.z;

    item.rotation[0] = transform.rotation.x;
    item.rotation[1] = transform.rotation.y;
    item.rotation[2] = transform.rotation.z;
    item.rotation[3] = transform.rotation.w;

    item.scale[0] = transform.scale.x;
    item.scale[1] = transform.scale.y;
    item.scale[2] = transform.scale.z;

    if (parent) {
      item.parentId = parent.id;
    } else {
      item.parentId = undefined;
    }
  }

  for (const [entity, name] of names) {
    const item = items.get(entity.id);
    if (!item) continue;

    item.name = name.value;
  }

  for (const extra of extras) {
    try {
      const item = items.get(extra.target);
      if (!item) continue;

      if (extra.key === "locked") {
        item.locked = JSON.parse(extra.value);
      }
    } catch {
      // Ignore invalid JSON
    }
  }

  // Destroy items that no longer exist
  for (const [id, item] of items.entries()) {
    if (!ids.includes(id)) {
      item.destroy();
    }
  }
}
