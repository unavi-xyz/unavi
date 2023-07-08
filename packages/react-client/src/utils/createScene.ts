import { Asset, CoreStore } from "lattice-engine/core";
import { OutlinePass } from "lattice-engine/postprocessing";
import {
  AmbientLight,
  DirectionalLight,
  GlobalTransform,
  Image,
  Name,
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

  const skyboxId = commands.spawn(true).addType(Asset).addType(Image).id;

  const name = new Name("root");

  const rootId = commands
    .spawn(true)
    .add(name)
    .addType(Transform)
    .addType(GlobalTransform).id;

  dropStruct(name);

  const sceneComponent = new Scene();
  sceneComponent.skyboxId = skyboxId;
  sceneComponent.rootId = rootId;

  const sceneId = commands
    .spawn(true)
    .add(sceneComponent)
    .addType(OutlinePass).id;

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
