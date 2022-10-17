import { EntityJSON } from "@wired-labs/engine";

import { useEditorStore } from "../store";

export class UpdateEntityAction {
  constructor(entityId: string, data: Partial<EntityJSON>) {
    const { engine, selectedId } = useEditorStore.getState();
    if (!engine) throw new Error("Engine not found");

    // This is a hack to force TransformControls to detach and reattach to the new object
    if (data.mesh !== undefined && selectedId === entityId) {
      useEditorStore.setState({ selectedId: null });
      setTimeout(() => useEditorStore.setState({ selectedId }));
    }

    // Update entity
    engine.scene.updateEntity(entityId, data);
  }
}

export function updateEntity(entityId: string, data: Partial<EntityJSON>) {
  return new UpdateEntityAction(entityId, data);
}
