import { EditNode, EditorEvent } from "@unavi/protocol";
import { Transform } from "houseki/scene";
import { TransformControls } from "houseki/transform";
import { Entity, Query, struct, SystemRes, u32 } from "thyseus";

import { useClientStore } from "../../client";
import { EditorId } from "../../client/components";

const PUBLISH_INTERVAL = 25;

@struct
class LocalRes {
  wasDragging: boolean = false;
  lastPublish: u32 = 0;
}

/**
 * Publish final transform, and on an interval while dragging
 */
export function publishTransformControlChanges(
  localRes: SystemRes<LocalRes>,
  transformControls: Query<TransformControls>,
  ids: Query<[Entity, EditorId]>,
  transforms: Query<[Entity, Transform]>
) {
  for (const control of transformControls) {
    if (!control.dragging && !localRes.wasDragging) continue;

    const changedDragState = localRes.wasDragging !== control.dragging;
    localRes.wasDragging = control.dragging;

    const now = performance.now();

    if (changedDragState) {
      localRes.lastPublish = now;
    }

    const shouldPublish =
      changedDragState || now - localRes.lastPublish > PUBLISH_INTERVAL;

    if (shouldPublish) {
      localRes.lastPublish = now;

      let id: string | undefined;
      for (const [entity, editorId] of ids) {
        if (entity.id === control.targetId) {
          id = editorId.value;
          break;
        }
      }
      if (!id) continue;

      let transform: Transform | undefined;
      for (const [entity, t] of transforms) {
        if (entity.id === control.targetId) {
          transform = t;
          break;
        }
      }
      if (!transform) continue;

      const editNode = EditNode.create({
        rotation: transform.rotation.toArray(),
        scale: transform.scale.toArray(),
        target: id,
        translation: transform.translation.toArray(),
      });
      const event = EditorEvent.create({
        event: { editNode, oneofKind: "editNode" },
      });
      useClientStore.getState().sendEditorEvent(event);
    }
  }
}
