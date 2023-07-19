import { WorldBuilder } from "thyseus";

import { ClientSchedules } from "../constants";
import { addMeshes } from "./systems/addMeshes";
import { addNodes } from "./systems/addNodes";
import { createTreeItems } from "./systems/createTreeItems";
import { editMeshes } from "./systems/editMeshes";
import { editNodes } from "./systems/editNodes";
import { enterEditMode } from "./systems/enterEditMode";
import { exitEditMode } from "./systems/exitEditMode";
import { sendExportEvent } from "./systems/sendExportEvent";
import { syncTransformTarget } from "./systems/syncTransformTarget";

export function editorPlugin(builder: WorldBuilder) {
  builder
    .addSystemsToSchedule(ClientSchedules.EnterEditMode, enterEditMode)
    .addSystemsToSchedule(ClientSchedules.ExitEditMode, exitEditMode)
    .addSystemsToSchedule(ClientSchedules.Export, sendExportEvent)
    .addSystems(
      addMeshes,
      addNodes,
      createTreeItems,
      editMeshes,
      editNodes,
      syncTransformTarget
    );
}
