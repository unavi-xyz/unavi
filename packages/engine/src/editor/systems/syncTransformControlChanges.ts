import { Transform } from "houseki/scene";
import { TransformControls } from "houseki/transform";
import { Entity, Query, struct, SystemRes, u32 } from "thyseus";

import { EditorId } from "../../client/components";
import { editNode } from "../actions";

const PUBLISH_INTERVAL = 25;

@struct
class LocalRes {
  wasDragging: boolean = false;
  lastSync: u32 = 0;
}

export function syncTransformControlChanges(
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
      localRes.lastSync = now;
    }

    const shouldSync =
      changedDragState || now - localRes.lastSync > PUBLISH_INTERVAL;

    if (shouldSync) {
      localRes.lastSync = now;

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

      editNode(id, {
        rotation: transform.rotation.toArray(),
        scale: transform.scale.toArray(),
        translation: transform.translation.toArray(),
      });
    }
  }
}
