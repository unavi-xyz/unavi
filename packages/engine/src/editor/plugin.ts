import { createControls, selectTarget } from "lattice-engine/transform";
import { run, WorldBuilder } from "thyseus";

import { EngineSchedules } from "../constants";
import { addMeshes } from "./systems/addMeshes";
import { addNodes } from "./systems/addNodes";
import { createTreeItemsPhysics } from "./systems/createTreeItemPhysics";
import { createTreeItems } from "./systems/createTreeItems";
import { editColliders } from "./systems/editColliders";
import { editExtras } from "./systems/editExtras";
import { editMeshes } from "./systems/editMeshes";
import { editNodes } from "./systems/editNodes";
import { editRigidBodies } from "./systems/editRigidBodies";
import { enterEditMode } from "./systems/enterEditMode";
import { exitEditMode } from "./systems/exitEditMode";
import { sendExportEvent } from "./systems/sendExportEvent";
import { syncTransformControls } from "./systems/syncTransformControls";

export function editorPlugin(builder: WorldBuilder) {
  builder
    .addSystemsToSchedule(EngineSchedules.EnterEditMode, enterEditMode)
    .addSystemsToSchedule(EngineSchedules.ExitEditMode, exitEditMode)
    .addSystemsToSchedule(EngineSchedules.Export, sendExportEvent)
    .addSystems(
      addMeshes,
      addNodes,
      createTreeItems,
      createTreeItemsPhysics,
      editExtras,
      editMeshes,
      editNodes,
      editRigidBodies,
      editColliders,
      run(syncTransformControls).after(selectTarget).before(createControls)
    );
}
