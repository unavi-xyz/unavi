import { OrbitControls } from "houseki/orbit";
import {
  GlobalTransform,
  PerspectiveCamera,
  SceneStruct,
  Transform,
} from "houseki/scene";
import { Commands } from "thyseus";

export function createOrbitControls(
  commands: Commands,
  sceneStruct: SceneStruct,
  translation: [number, number, number] = [0, 2, 4]
) {
  const transform = new Transform(translation);

  const camera = commands
    .spawn(true)
    .add(transform)
    .addType(GlobalTransform)
    .addType(PerspectiveCamera)
    .addType(OrbitControls);

  sceneStruct.activeCamera = camera.id;
}
