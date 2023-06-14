import { Asset, CoreStore } from "lattice-engine/core";
import { GlobalTransform, Parent, SceneStruct, Transform } from "lattice-engine/scene";
import { Commands, dropStruct, Mut, Res } from "thyseus";

import { WorldJson } from "../components";
import { createOrbitControls } from "../utils/createOrbitControls";
import { createScene } from "../utils/createScene";

export function initApp(
  commands: Commands,
  coreStore: Res<Mut<CoreStore>>,
  sceneStruct: Res<Mut<SceneStruct>>
) {
  coreStore.canvas = document.querySelector("canvas");

  createOrbitControls(commands, sceneStruct);

  const { root } = createScene(commands, sceneStruct);

  // World
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
