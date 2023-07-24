import { TransformControls } from "lattice-engine/transform";
import { Mut, Query } from "thyseus";

import { useSceneStore } from "../sceneStore";

let lastTransformTarget: bigint | undefined;

export function syncTransformTarget(
  transformControls: Query<Mut<TransformControls>>
) {
  for (const transformControl of transformControls) {
    const targetId = transformControl.targetId;

    const { sceneTreeId, rootId, items } = useSceneStore.getState();

    // Set UI from transform controls
    if (targetId !== lastTransformTarget) {
      lastTransformTarget = targetId;

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

      // Ignore if currentId is locked
      // if (items.get(currentId)?.locked) {
      //   useSceneStore.setState({ selectedId: undefined });
      //   transformControl.targetId = 0n;
      //   continue;
      // }

      useSceneStore.setState({ selectedId: currentId });
      transformControl.targetId = currentId;
    } else {
      // Set transform controls from UI
      const newTarget = useSceneStore.getState().selectedId ?? 0n;
      transformControl.targetId = newTarget;
      lastTransformTarget = newTarget;

      // Disable transform controls if target is locked
      const locked = items.get(newTarget)?.locked;
      transformControl.enabled = !locked;
    }
  }
}
