import { ExportGltf } from "houseki/gltf";
import { SceneView } from "houseki/scene";
import { Entity, EventWriter, Query, With } from "thyseus";

import { WorldJson } from "../../client/components";

export function sendExportEvent(
  writer: EventWriter<ExportGltf>,
  views: Query<Entity, [With<SceneView>, With<WorldJson>]>
) {
  for (const entity of views) {
    const e = new ExportGltf();
    e.scene = entity.id;
    e.binary = true;
    writer.create(e);
  }
}
