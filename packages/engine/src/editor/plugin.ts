import { createControls, selectTarget } from "houseki/transform";
import { run, WorldBuilder } from "thyseus";

import { EngineSchedules } from "../constants";
import { createMeshes } from "./systems/createMeshes";
import { createNodes } from "./systems/createNodes";
import { enterEditMode } from "./systems/enterEditMode";
import { exitEditMode } from "./systems/exitEditMode";
import { initSyncedStore } from "./systems/initSyncedStore";
import { sendExportEvent } from "./systems/sendExportEvent";
import { setEntityIds } from "./systems/setEntityIds";
import { syncTransformControlChanges } from "./systems/syncTransformControlChanges";
import { syncTransformControlTarget } from "./systems/syncTransformControlTarget";

export function editorPlugin(builder: WorldBuilder) {
  builder
    .addSystemsToSchedule(EngineSchedules.EnterEditMode, enterEditMode)
    .addSystemsToSchedule(EngineSchedules.ExitEditMode, exitEditMode)
    .addSystemsToSchedule(EngineSchedules.Export, sendExportEvent)
    .addSystemsToSchedule(EngineSchedules.EnterEditMode, initSyncedStore)
    .addSystems(
      setEntityIds,
      createNodes,
      createMeshes,
      syncTransformControlChanges,
      run(syncTransformControlTarget).after(selectTarget).before(createControls)
    );
}
