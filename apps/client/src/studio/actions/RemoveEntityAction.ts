import { useStudioStore } from "../store";
import { updateTree } from "../utils/tree";

export class RemoveEntityAction {
  constructor(entityId: string) {
    // Add the entity to the tree
    const { tree, engine } = useStudioStore.getState();
    delete tree[entityId];
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
