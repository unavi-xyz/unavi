import { Asset, CoreStore } from "lattice-engine/core";
import {
  AmbientLight,
  DirectionalLight,
  GlobalTransform,
  Image,
  Parent,
  Scene,
  SceneStruct,
  ShadowMap,
  Transform,
} from "lattice-engine/scene";
import { Commands, dropStruct } from "thyseus";

export function createScene(
  commands: Commands,
  coreStore: CoreStore,
  sceneStruct: SceneStruct,
  shadowResolution = 2048,
  shadowArea = 8
) {
  const canvas = document.querySelector("canvas");
  coreStore.canvas = canvas;

  const asset = new Asset("/Skybox.jpg", "image/jpeg");
  const image = new Image(true);

  const skyboxId = commands.spawn().add(asset).add(image).id;

  dropStruct(asset);
  dropStruct(image);

  const rootId = commands
    .spawn(true)
    .addType(Transform)
    .addType(GlobalTransform).id;

  const sceneComponent = new Scene();
  sceneComponent.skyboxId = skyboxId;
  sceneComponent.rootId = rootId;

  const sceneId = commands.spawn(true).add(sceneComponent).id;

  dropStruct(sceneComponent);

  sceneStruct.activeScene = sceneId;

  const parent = new Parent(sceneId);
  commands.getById(rootId).add(parent);

  const ambient = new AmbientLight([1, 1, 1], 0.25);
  commands
    .spawn(true)
    .add(ambient)
    .addType(Transform)
    .addType(GlobalTransform)
    .add(parent);
  dropStruct(ambient);

  const directionalComponent = new DirectionalLight([1, 1, 1], 0.75);
  const transform = new Transform([0, 30, 0]);

  const directional = commands
    .spawn(true)
    .add(directionalComponent)
    .add(transform)
    .addType(GlobalTransform)
    .add(parent);

  dropStruct(transform);
  dropStruct(directionalComponent);
  dropStruct(parent);

  if (shadowResolution > 0) {
    const shadowMap = new ShadowMap(
      shadowResolution,
      -shadowArea,
      shadowArea,
      shadowArea,
      -shadowArea,
      0.1,
      50
    );
    directional.add(shadowMap);
    dropStruct(shadowMap);
  }

  return { rootId, sceneId };
}
