import { useEditorStore } from "../store";

export class MoveEntityAction {
  constructor(entityId: string, parentId: string | null, index?: number) {
    const { scene, engine } = useEditorStore.getState();
    const entity = scene.entities[entityId];

    // Remove from old parent
    if (entity.parent) {
      const oldParent = scene.entities[entity.parent];
      oldParent.children = oldParent.children.filter((id) => id !== entityId);

      oldParent.children = [...oldParent.children];
    }

    // Set new parent
    entity.parent = parentId;

    // Add to new parent
    if (parentId) {
      const newParent = scene.entities[parentId];
      if (index === undefined) newParent.children.push(entityId);
      else newParent.children.splice(index, 0, entityId);

      newParent.children = [...newParent.children];
    }

    // Update scene
    useEditorStore.setState({ scene });

    // Update engine
    if (engine) engine.renderThread.moveEntity(entityId, parentId);
  }
}

export function moveEntity(
  entityId: string,
  parentId: string | null,
  index?: number
) {
  return new MoveEntityAction(entityId, parentId, index);
}
