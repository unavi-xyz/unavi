import { OrbitControls } from "lattice-engine/orbit";
import {
  GlobalTransform,
  PerspectiveCamera,
  SceneStruct,
  Transform,
} from "lattice-engine/scene";
import { Commands, dropStruct } from "thyseus";

export function createOrbitControls(
  commands: Commands,
  sceneStruct: SceneStruct,
  translation: [number, number, number] = [0, 2, 4]
) {
  const transform = new Transform(translation);

  const camera = commands
    .spawn()
    .add(transform)
    .addType(GlobalTransform)
    .addType(PerspectiveCamera)
    .addType(OrbitControls);

  dropStruct(transform);

  sceneStruct.activeCamera = camera.id;
}
