import { EditId_Type } from "@unavi/protocol";
import { GltfInfo } from "houseki/gltf";
import { Commands, Entity, EventReader, Mut, Query } from "thyseus";

import { EditorId } from "../../client/components";
import { EditId } from "../events";

export function editIds(
  commands: Commands,
  reader: EventReader<EditId>,
  infos: Query<GltfInfo>,
  withIds: Query<[Entity, Mut<EditorId>]>
) {
  if (!reader.length) return;

  for (const e of reader) {
    for (const info of infos) {
      let targetId: bigint | undefined;

      switch (e.type) {
        case EditId_Type.SCENE: {
          targetId = info.scenes[e.index];
          break;
        }

        case EditId_Type.NODE: {
          targetId = info.nodes[e.index];
          break;
        }

        case EditId_Type.MESH: {
          targetId = info.meshes[e.index];
          break;
        }

        case EditId_Type.MATERIAL: {
          targetId = info.materials[e.index];
          break;
        }
      }

      if (!targetId) continue;

      let found = false;

      for (const [entity, id] of withIds) {
        if (entity.id === targetId) {
          id.value = e.id;
          found = true;
        }
      }

      if (!found) {
        commands.getById(targetId).add(new EditorId(e.id));
      }
    }
  }

  reader.clear();
}
