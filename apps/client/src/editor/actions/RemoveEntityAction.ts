import { useEditorStore } from "../store";

export class RemoveEntityAction {
  constructor(entityId: string) {
    const { engine } = useEditorStore.getState();
    if (!engine) throw new Error("Engine not found");

    // Remove entity
    engine.scene.removeEntity(entityId);
  }
}

export function removeEntity(entityId: string) {
  return new RemoveEntityAction(entityId);
}
