import { TransformControls, TransformMode } from "lattice-engine/transform";
import { Mut, Query } from "thyseus";

import { editorStore } from "../store";
import { Tool } from "../types";

let lastTransformTarget: bigint | undefined;

export function syncTransformControls(
  transformControls: Query<Mut<TransformControls>>
) {
  for (const controls of transformControls) {
    const targetId = controls.targetId;

    const sceneTreeId = editorStore.get(editorStore.sceneTreeId);
    const selectedId = editorStore.get(editorStore.selectedId);
    const rootId = editorStore.get(editorStore.rootId);
    const items = editorStore.get(editorStore.items);
    const tool = editorStore.get(editorStore.tool);

    switch (tool) {
      case Tool.Translate: {
        controls.mode = TransformMode.Translate;
        break;
      }

      case Tool.Rotate: {
        controls.mode = TransformMode.Rotate;
        break;
      }

      case Tool.Scale: {
        controls.mode = TransformMode.Scale;
        break;
      }
    }

    const uiId = selectedId ?? 0n;

    if (targetId !== lastTransformTarget) {
      // Set UI from transform controls
      lastTransformTarget = targetId;

      const usedId = sceneTreeId ?? rootId;
      if (!usedId) continue;

      // Continue up tree until we reach a child of usedId
      let newTargetId = targetId;

      while (newTargetId !== usedId) {
        const parentId: bigint | undefined = items.get(newTargetId)?.parentId;
        if (!parentId) break;

        if (parentId === usedId) {
          // We've reached a child of usedId
          break;
        }

        newTargetId = parentId;
      }

      // If locked, do not select it
      if (items.get(newTargetId)?.locked) {
        newTargetId = 0n;
      }

      setTransformTarget(controls, newTargetId);
    } else if (targetId !== uiId) {
      // Set transform controls from UI
      lastTransformTarget = uiId;
      setTransformTarget(controls, uiId);
    }
  }
}

function setTransformTarget(controls: TransformControls, targetId: bigint) {
  const items = editorStore.get(editorStore.items);

  controls.targetId = targetId;

  const selectedId = targetId === 0n ? undefined : targetId;
  editorStore.set(editorStore.selectedId, selectedId);

  // Disable transform controls if target is locked
  const locked = items.get(targetId)?.locked;
  controls.enabled = !locked;
}
