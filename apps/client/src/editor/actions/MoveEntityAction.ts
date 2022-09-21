import { useEditorStore } from "../store";

export class MoveEntityAction {
  constructor(entityId: string, parentId: string, index?: number) {
    const { engine, getEntity } = useEditorStore.getState();
    const entity = getEntity(entityId);
    if (!engine) throw new Error("Engine not found");
    if (!entity) throw new Error("Entity not found");

    // Set new parent
    engine.scene.updateEntity(entityId, { parentId });

    // Place entity at index
    if (index !== undefined) {
      const parent = getEntity(parentId);
      if (!parent) throw new Error("Parent not found");

      const sorted = parent.childrenIds.filter((id) => id !== entityId);
      sorted.splice(index, 0, entityId);

      parent.childrenIds$.next(sorted);
    }
  }
}

export function moveEntity(entityId: string, parentId: string, index?: number) {
  return new MoveEntityAction(entityId, parentId, index);
}
