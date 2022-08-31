import { useStudioStore } from "../store";
import { updateTree } from "../utils/tree";

export class MoveEntityAction {
  constructor(entityId: string, parentId: string | null) {
    const parent = parentId === "root" ? null : parentId;

    // Update tree
    const { tree, engine } = useStudioStore.getState();
    const entity = tree[entityId];
    entity.parent = parent;
    useStudioStore.setState({ tree });

    // Update UI
    updateTree();

    // Update engine
    if (engine) engine.renderThread.moveEntity(entityId, parent);
  }
}

export function moveEntity(entityId: string, parentId: string | null) {
  return new MoveEntityAction(entityId, parentId);
}
