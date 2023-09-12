import { EditId_Type } from "@unavi/protocol";
import { GltfInfo } from "houseki/gltf";
import { SceneStruct } from "houseki/scene";
import { Entity, Query, Res, Without } from "thyseus";

import { useClientStore } from "../../client";
import { EditorId } from "../../client/components";
import { nanoidShort } from "../../client/utils/nanoid";

export function initIds(
  infos: Query<GltfInfo>,
  sceneStruct: Res<SceneStruct>,
  withoutIds: Query<Entity, Without<EditorId>>
) {
  const entities: bigint[] = [];

  for (const entity of withoutIds) {
    entities.push(entity.id);
  }

  for (const info of infos) {
    processEntity(
      sceneStruct.activeScene,
      EditId_Type.SCENE,
      info.defaultScene,
      entities
    );

    info.nodes.forEach((entityId, i) =>
      processEntity(entityId, EditId_Type.NODE, i, entities)
    );

    info.meshes.forEach((entityId, i) => {
      processEntity(entityId, EditId_Type.MESH, i, entities);
    });

    info.materials.forEach((entityId, i) =>
      processEntity(entityId, EditId_Type.MATERIAL, i, entities)
    );
  }
}

function processEntity(
  entityId: bigint,
  type: EditId_Type,
  index: number,
  entities: bigint[]
) {
  if (!entities.includes(entityId)) return;

  const id = nanoidShort();

  useClientStore.getState().sendEditorEvent({
    event: { editId: { id, index, type }, oneofKind: "editId" },
  });
}
