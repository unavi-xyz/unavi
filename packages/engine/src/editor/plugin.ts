import { createControls, selectTarget } from "houseki/transform";
import { run, WorldBuilder } from "thyseus";

import { EngineSchedules } from "../constants";
import { createMeshes } from "./systems/createMeshes";
import { createNodes } from "./systems/createNodes";
import { createScenes } from "./systems/createScenes";
import { enterEditMode } from "./systems/enterEditMode";
import { exitEditMode } from "./systems/exitEditMode";
import { initSyncedStore } from "./systems/initSyncedStore";
import { sendExportEvent } from "./systems/sendExportEvent";
import { setEntityIds } from "./systems/setEntityIds";
import { syncTransformControlChanges } from "./systems/syncTransformControlChanges";
import { syncTransformControlTarget } from "./systems/syncTransformControlTarget";

export function editorPlugin(builder: WorldBuilder) {
  builder
    .addSystemsToSchedule(
      EngineSchedules.EnterEditMode,
      initSyncedStore,
      enterEditMode
    )
    .addSystemsToSchedule(EngineSchedules.ExitEditMode, exitEditMode)
    .addSystemsToSchedule(EngineSchedules.Export, sendExportEvent)
    .addSystems(
      setEntityIds,
      createScenes,
      createNodes,
      createMeshes,
      syncTransformControlChanges,
      run(syncTransformControlTarget).after(selectTarget).before(createControls)
    );
}
