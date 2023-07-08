import { Asset, CoreStore } from "lattice-engine/core";
import { InputStruct } from "lattice-engine/input";
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
  inputStruct: Res<Mut<InputStruct>>
) {
  coreStore.canvas = document.querySelector("canvas");

  const { rootId } = createScene(commands, coreStore, sceneStruct);

  createPlayerControls([0, 4, 0], rootId, commands, sceneStruct, inputStruct);

  const parent = new Parent(rootId);

  commands
    .spawn(true)
    .add(parent)
    .addType(Transform)
    .addType(GlobalTransform)
    .addType(Asset)
    .addType(WorldJson);

  dropStruct(parent);
}
