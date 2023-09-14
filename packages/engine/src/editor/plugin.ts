import { createControls, selectTarget } from "houseki/transform";
import { run, WorldBuilder } from "thyseus";

import { EngineSchedules } from "../constants";
import { addMeshPrimitives } from "./systems/addMeshPrimitives";
import { addNodes } from "./systems/addNodes";
import { createNodeItemPhysics } from "./systems/createNodeItemPhysics";
import { createNodeItems } from "./systems/createNodeItems";
import { editColliders } from "./systems/editColliders";
import { editExtras } from "./systems/editExtras";
import { editIds } from "./systems/editIds";
import { editMeshPrimitives } from "./systems/editMeshPrimitives";
import { editNodes } from "./systems/editNodes";
import { editRigidBodies } from "./systems/editRigidBodies";
import { enterEditMode } from "./systems/enterEditMode";
import { exitEditMode } from "./systems/exitEditMode";
import { initIds } from "./systems/initIds";
import { sendExportEvent } from "./systems/sendExportEvent";
import { syncTransformControls } from "./systems/syncTransformControls";

export function editorPlugin(builder: WorldBuilder) {
  builder
    .addSystemsToSchedule(EngineSchedules.EnterEditMode, enterEditMode)
    .addSystemsToSchedule(EngineSchedules.ExitEditMode, exitEditMode)
    .addSystemsToSchedule(EngineSchedules.Export, sendExportEvent)
    .addSystemsToSchedule(EngineSchedules.EnterEditMode, initIds)
    .addSystems(
      addMeshPrimitives,
      addNodes,
      createNodeItems,
      createNodeItemPhysics,
      editExtras,
      editMeshPrimitives,
      editIds,
      editNodes,
      editRigidBodies,
      editColliders,
      run(syncTransformControls).after(selectTarget).before(createControls)
    );
}
