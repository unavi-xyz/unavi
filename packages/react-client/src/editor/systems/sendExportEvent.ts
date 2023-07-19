import { ExportGltf } from "lattice-engine/gltf";
import { SceneStruct } from "lattice-engine/scene";
import { EventWriter, Res } from "thyseus";

export function sendExportEvent(
  writer: EventWriter<ExportGltf>,
  sceneStruct: Res<SceneStruct>
) {
  const event = writer.create();
  event.scene = sceneStruct.activeScene;
  event.binary = true;
}
