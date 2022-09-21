import { EntityJSON } from "@wired-labs/engine";

import { useEditorStore } from "../store";

export class UpdateEntityAction {
  constructor(entityId: string, data: Partial<EntityJSON>) {
    const { engine } = useEditorStore.getState();
    if (!engine) throw new Error("Engine not found");

    // Update entity
    engine.scene.updateEntity(entityId, data);
  }
}

export function updateEntity(entityId: string, data: Partial<EntityJSON>) {
  return new UpdateEntityAction(entityId, data);
}
