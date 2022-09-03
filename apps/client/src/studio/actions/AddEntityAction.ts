import { Entity } from "@wired-labs/engine";

import { useStudioStore } from "../store";
import { updateTree } from "../utils/tree";

export class AddEntityAction {
  constructor(entity: Entity) {
    // Add the entity to the tree
    const { tree, engine } = useStudioStore.getState();
    tree[entity.id] = entity;
    useStudioStore.setState({ tree });

    // Update UI
    updateTree();

    // Update engine
    if (engine) engine.renderThread.addEntity(entity);
  }
}

export function addEntity(entity: Entity) {
  return new AddEntityAction(entity);
}
