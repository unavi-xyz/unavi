import { useStudioStore } from "../store";

export class MoveEntityAction {
  constructor(entityId: string, parentId: string | null, index?: number) {
    const { tree, engine } = useStudioStore.getState();
    const entity = tree[entityId];

    // Remove from old parent
    if (entity.parent) {
      const oldParent = tree[entity.parent];
      oldParent.children = oldParent.children.filter((id) => id !== entityId);

      oldParent.children = [...oldParent.children];
    }

    // Set new parent
    entity.parent = parentId;

    // Add to new parent
    if (parentId) {
      const newParent = tree[parentId];
      if (index === undefined) newParent.children.push(entityId);
      else newParent.children.splice(index, 0, entityId);

      newParent.children = [...newParent.children];
    }

    // Update tree
    useStudioStore.setState({ tree });

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
