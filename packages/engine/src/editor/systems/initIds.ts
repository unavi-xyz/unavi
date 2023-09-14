import { EditId_Type } from "@unavi/protocol";
import { GltfInfo } from "houseki/gltf";
import { SceneStruct } from "houseki/scene";
import { Entity, Query, Res, With } from "thyseus";

import { useClientStore } from "../../client";
import { EditorId } from "../../client/components";
import { nanoidShort } from "../../client/utils/nanoid";

export function initIds(
  infos: Query<GltfInfo>,
  sceneStruct: Res<SceneStruct>,
  withIds: Query<Entity, With<EditorId>>
) {
  const processed: bigint[] = [];

  for (const entity of withIds) {
    processed.push(entity.id);
  }

  for (const info of infos) {
    processEntity(
      sceneStruct.activeScene,
      EditId_Type.SCENE,
      info.defaultScene,
      processed
    );

    info.nodes.forEach((entityId, i) => {
      processEntity(entityId, EditId_Type.NODE, i, processed);
    });

    info.meshPrimitives.forEach((entityId, i) => {
      processEntity(entityId, EditId_Type.PRIMITIVE, i, processed);
    });

    info.materials.forEach((entityId, i) => {
      processEntity(entityId, EditId_Type.MATERIAL, i, processed);
    });
  }
}

function processEntity(
  entityId: bigint,
  type: EditId_Type,
  index: number,
  processed: bigint[]
) {
  if (processed.includes(entityId)) return;

  processed.push(entityId);

  const id = nanoidShort();

  useClientStore.getState().sendEditorEvent({
    event: { editId: { id, index, type }, oneofKind: "editId" },
  });
}
