import { useStudioStore } from "../store";
import { updateTree } from "../utils/tree";

export class RemoveEntityAction {
  constructor(entityId: string) {
    const { tree, engine } = useStudioStore.getState();

    // Remove from parent
    const entity = tree[entityId];
    if (entity.parent) {
      const parent = tree[entity.parent];
      parent.children = parent.children.filter((id) => id !== entityId);
    }

    // Remove entity
    delete tree[entityId];

    // Update tree
    useStudioStore.setState({ tree });

    // Update UI
    updateTree();

    // Update engine
    if (engine) engine.renderThread.removeEntity(entityId);
  }
}

export function removeEntity(entityId: string) {
  return new RemoveEntityAction(entityId);
}
