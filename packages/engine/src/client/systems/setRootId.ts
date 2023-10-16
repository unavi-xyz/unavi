import { SceneView } from "houseki/scene";
import { Entity, Query, With } from "thyseus";

import { useSceneStore } from "../../editor";
import { EditorId, WorldJson } from "../components";

export function setRootId(
  views: Query<SceneView, With<WorldJson>>,
  ids: Query<[Entity, EditorId]>
) {
  for (const view of views) {
    for (const [ent, editorId] of ids) {
      if (ent.id === view.active) {
        useSceneStore.setState({ rootId: editorId.value });
      }
    }
  }
}
