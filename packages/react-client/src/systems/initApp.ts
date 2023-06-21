import { Asset, CoreStore } from "lattice-engine/core";
import { InputStruct } from "lattice-engine/input";
import { PhysicsConfig } from "lattice-engine/physics";
import {
  GlobalTransform,
  Parent,
  SceneStruct,
  Transform,
} from "lattice-engine/scene";
import { Commands, dropStruct, Mut, Res } from "thyseus";

import { WorldJson } from "../components";
import { createPlayerControls } from "../utils/createPlayerControls";
import { createScene } from "../utils/createScene";

export function initApp(
  commands: Commands,
  coreStore: Res<Mut<CoreStore>>,
  sceneStruct: Res<Mut<SceneStruct>>,
  inputStruct: Res<Mut<InputStruct>>,
  physicsConfig: Res<Mut<PhysicsConfig>>
) {
  physicsConfig.debug = true;

  coreStore.canvas = document.querySelector("canvas");

  const { root } = createScene(commands, sceneStruct);

  createPlayerControls(root, commands, sceneStruct, inputStruct);

  const parent = new Parent(root);

  commands
    .spawn()
    .add(parent)
    .addType(Transform)
    .addType(GlobalTransform)
    .addType(Asset)
    .addType(WorldJson);

  dropStruct(parent);
}
