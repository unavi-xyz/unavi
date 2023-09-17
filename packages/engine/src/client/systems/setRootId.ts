import { SceneView } from "houseki/gltf";
import { Entity, Query } from "thyseus";

import { useSceneStore } from "../../editor";
import { EditorId } from "../components";

export function setRootId(
  views: Query<SceneView>,
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
