import { Asset, CoreStore } from "houseki/core";
import { CascadingShadowMaps } from "houseki/csm";
import { InputStruct } from "houseki/input";
import { Name, SceneStruct } from "houseki/scene";
import { Commands, Mut, Res } from "thyseus";

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
    [0, 4, 0],
    sceneId,
    commands,
    sceneStruct
  );

  const csm = new CascadingShadowMaps();
  csm.shadowMapSize = 4096;
  csm.far = 40;
  commands.getById(cameraId).add(csm);

  const name = new Name("root");
  commands.getById(rootId).add(name).addType(Asset).addType(WorldJson);
}
