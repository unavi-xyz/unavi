import {
  GlobalTransform,
  Mesh,
  Name,
  Parent,
  Scene,
  Transform,
} from "houseki/scene";
import { Commands, Entity, Mut, Query, Without } from "thyseus";

import { EditorId } from "../../client/components";
import { getEntityId } from "../entities";
import { syncedStore } from "../store";

export function createNodes(
  commands: Commands,
  nodes: Query<
    [Entity, EditorId, Mut<Name>, Mut<Transform>, Mut<Parent>],
    Without<Scene>
  >,
  meshes: Query<[EditorId, Mut<Mesh>]>
) {
  const ids: string[] = [];

  for (const [entity, id, name, transform, parent] of nodes) {
    ids.push(id.value);

    const node = syncedStore.nodes[id.value];

    if (!node) {
      // Node was deleted, remove from ecs
      commands.get(entity).despawn();
    } else {
      // Sync data
      name.value = node.name;

      transform.translation.fromArray(node.translation);
      transform.rotation.fromArray(node.rotation);
      transform.scale.fromArray(node.scale);

      if (node.parentId) {
        parent.id = getEntityId(node.parentId) ?? 0n;
      } else {
        const hasParent = false;

        if (!hasParent) {
          parent.id = 0n;
        }
      }

      for (const [meshId, mesh] of meshes) {
        if (node.meshId === meshId.value) {
          mesh.parentId = entity.id;
        } else if (mesh.parentId === entity.id) {
          mesh.parentId = 0n;
        }
      }
    }
  }

  // Create new nodes
  for (const id of Object.keys(syncedStore.nodes)) {
    if (!ids.includes(id)) {
      commands
        .spawn(true)
        .add(new EditorId(id))
        .addType(Name)
        .addType(Transform)
        .addType(GlobalTransform)
        .addType(Parent);
    }
  }
}
