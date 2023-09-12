import { Scene, SceneStruct } from "houseki/scene";
import { Entity, Query, Res } from "thyseus";

import { useClientStore } from "../clientStore";
import { EditorId } from "../components";

export function setRootId(
  sceneStruct: Res<SceneStruct>,
  scenes: Query<[Entity, Scene]>,
  ids: Query<[Entity, EditorId]>
) {
  for (const [entity, scene] of scenes) {
    if (sceneStruct.activeScene !== entity.id) continue;

    for (const [entity, id] of ids) {
      if (scene.rootId !== entity.id) continue;

      useClientStore.setState({ rootId: id.value });
    }
  }
}
