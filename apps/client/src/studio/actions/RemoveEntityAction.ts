import { useStudioStore } from "../store";

export class RemoveEntityAction {
  constructor(entityId: string) {
    const { tree, engine } = useStudioStore.getState();

    // Remove from parent
    const entity = tree[entityId];
    if (entity.parent) {
      const parent = tree[entity.parent];
      parent.children = parent.children.filter((id) => id !== entityId);
      parent.children = [...parent.children];
    }

    // Remove entity
    delete tree[entityId];

    // Update tree
    useStudioStore.setState({ tree });

    // Update engine
    if (engine) engine.renderThread.removeEntity(entityId);
  }
}

export function removeEntity(entityId: string) {
  return new RemoveEntityAction(entityId);
}
