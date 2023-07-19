import { WorldBuilder } from "thyseus";

import { ClientSchedules } from "../constants";
import { addMeshes } from "./systems/addMeshes";
import { addNodes } from "./systems/addNodes";
import { createTreeItems } from "./systems/createTreeItems";
import { enterEditMode } from "./systems/enterEditMode";
import { exitEditMode } from "./systems/exitEditMode";
import { sendExportEvent } from "./systems/sendExportEvent";

export function editorPlugin(builder: WorldBuilder) {
  builder
    .addSystemsToSchedule(ClientSchedules.EnterEditMode, enterEditMode)
    .addSystemsToSchedule(ClientSchedules.ExitEditMode, exitEditMode)
    .addSystemsToSchedule(ClientSchedules.Export, sendExportEvent)
    .addSystems(addMeshes, addNodes, createTreeItems);
}
