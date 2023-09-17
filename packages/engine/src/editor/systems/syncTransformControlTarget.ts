import { TransformControls, TransformMode } from "houseki/transform";
import { Mut, Query, struct, SystemRes, u64 } from "thyseus";

import { getEntityId, getId, getNodeByEntityId } from "../entities";
import { useSceneStore } from "../sceneStore";
import { Tool } from "../types";

@struct
class LocalStore {
  lastTransformTarget: u64 = 0n;
}

export function syncTransformControlTarget(
  localStore: SystemRes<LocalStore>,
  transformControls: Query<Mut<TransformControls>>
) {
  for (const controls of transformControls) {
    const targetId = controls.targetId;

    const { sceneTreeId, selectedId, rootId, tool } = useSceneStore.getState();

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

    const selectedEntityId = selectedId ? getEntityId(selectedId) : undefined;
    const uiId = selectedEntityId ?? 0n;

    if (targetId !== localStore.lastTransformTarget) {
      // Set UI from transform controls
      localStore.lastTransformTarget = targetId;

      const usedId = sceneTreeId ?? rootId;
      if (!usedId) continue;

      const usedEntityId = getEntityId(usedId);
      if (!usedEntityId) continue;

      // Continue up tree until we reach a child of usedId
      let newTargetId = targetId;

      while (newTargetId !== usedEntityId) {
        const newTarget = getNodeByEntityId(newTargetId);
        if (!newTarget?.parentId) break;

        const parentId = getEntityId(newTarget.parentId);
        if (!parentId) break;

        if (parentId === usedEntityId) {
          // We've reached a child of usedId
          break;
        }

        newTargetId = parentId;
      }

      const newTarget = getNodeByEntityId(newTargetId);
      if (!newTarget) continue;

      // If locked, do not select it
      if (newTarget.locked) {
        newTargetId = 0n;
      }

      setTransformTarget(controls, newTargetId);
    } else if (targetId !== uiId) {
      // Set transform controls from UI
      localStore.lastTransformTarget = uiId;
      setTransformTarget(controls, uiId);
    }
  }
}

function setTransformTarget(controls: TransformControls, targetId: bigint) {
  controls.targetId = targetId;

  const selectedEntityId = targetId === 0n ? undefined : targetId;
  const selectedId = selectedEntityId ? getId(selectedEntityId) : undefined;
  useSceneStore.setState({ selectedId });

  // Disable transform controls if target is locked
  const target = getNodeByEntityId(targetId);
  if (!target) return;

  controls.enabled = !target.locked;
}
