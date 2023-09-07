import { ExportGltf } from "houseki/gltf";
import { SceneStruct } from "houseki/scene";
import { EventWriter, Res } from "thyseus";

export function sendExportEvent(
  writer: EventWriter<ExportGltf>,
  sceneStruct: Res<SceneStruct>
) {
  const e = new ExportGltf();
  e.scene = sceneStruct.activeScene;
  e.binary = true;
  writer.create(e);
}
