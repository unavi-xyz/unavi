import { TransformControls } from "lattice-engine/transform";
import { Mut, Query } from "thyseus";

import { useSceneStore } from "../sceneStore";

let lastTransformTarget: bigint | undefined;

export function syncTransformTarget(
  transformControls: Query<Mut<TransformControls>>
) {
  for (const transformControl of transformControls) {
    const targetId = transformControl.targetId;

    if (targetId !== lastTransformTarget) {
      // Set UI from transform controls
      const { sceneTreeId, rootId, items } = useSceneStore.getState();
      const usedId = sceneTreeId ?? rootId;
      if (!usedId) continue;

      // Continue up tree until we reach a child of usedId
      let currentId = targetId;

      while (currentId !== usedId) {
        const parentId: bigint | undefined = items.get(currentId)?.parentId;
        if (!parentId) break;

        if (parentId === usedId) {
          // We've reached a child of usedId
          break;
        }

        currentId = parentId;
      }

      useSceneStore.setState({ selectedId: currentId });
    } else {
      // Set transform controls from UI
      transformControl.targetId = useSceneStore.getState().selectedId ?? 0n;
    }

    lastTransformTarget = targetId;
  }
}
