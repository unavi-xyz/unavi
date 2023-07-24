import { Asset, CoreStore } from "lattice-engine/core";
import { OutlinePass } from "lattice-engine/postprocessing";
import {
  GlobalTransform,
  Image,
  Parent,
  Scene,
  SceneStruct,
  Skybox,
  Transform,
} from "lattice-engine/scene";
import { Commands, dropStruct } from "thyseus";

export function createScene(
  commands: Commands,
  coreStore: CoreStore,
  sceneStruct: SceneStruct
) {
  const canvas = document.querySelector("canvas");
  coreStore.canvas = canvas;

  const rootId = commands
    .spawn(true)
    .addType(Transform)
    .addType(GlobalTransform).id;

  const skyboxId = commands.spawn(true).addType(Asset).addType(Image).id;
  const skybox = new Skybox();
  skybox.imageId = skyboxId;

  const sceneComponent = new Scene();
  sceneComponent.rootId = rootId;

  const sceneId = commands
    .spawn(true)
    .add(sceneComponent)
    .add(skybox)
    .addType(OutlinePass).id;

  dropStruct(sceneComponent);
  dropStruct(skybox);

  sceneStruct.activeScene = sceneId;

  const parent = new Parent(sceneId);
  commands.getById(rootId).add(parent);

  return { rootId, sceneId };
}
