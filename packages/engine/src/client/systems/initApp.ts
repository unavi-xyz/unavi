import { Asset, CoreStore } from "lattice-engine/core";
import { CascadingShadowMaps } from "lattice-engine/csm";
import { InputStruct } from "lattice-engine/input";
import { Name, SceneStruct } from "lattice-engine/scene";
import { Commands, dropStruct, Mut, Res } from "thyseus";

import { ENABLE_POINTER_LOCK } from "../../constants";
import { WorldJson } from "../components";
import { createPlayerControls } from "../utils/createPlayerControls";
import { createScene } from "../utils/createScene";

export function initApp(
  commands: Commands,
  coreStore: Res<Mut<CoreStore>>,
  sceneStruct: Res<Mut<SceneStruct>>,
  inputStruct: Res<Mut<InputStruct>>
) {
  inputStruct.enablePointerLock = ENABLE_POINTER_LOCK;

  coreStore.canvas = document.querySelector("canvas");

  const { rootId, sceneId } = createScene(commands, coreStore, sceneStruct);
  const cameraId = createPlayerControls(
    [0, 8, 0],
    sceneId,
    commands,
    sceneStruct
  );

  const csm = new CascadingShadowMaps();
  csm.shadowMapSize = 4096;
  csm.far = 40;
  commands.getById(cameraId).add(csm);
  dropStruct(csm);

  const name = new Name("root");
  commands.getById(rootId).add(name).addType(Asset).addType(WorldJson);
  dropStruct(name);
}
