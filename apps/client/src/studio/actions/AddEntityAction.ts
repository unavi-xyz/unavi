import { Entity } from "@wired-labs/engine";

import { useStudioStore } from "../store";
import { updateTree } from "../utils/tree";

export class AddEntityAction {
  constructor(entity: Entity) {
    const { tree, engine } = useStudioStore.getState();

    // Add entity
    tree[entity.id] = entity;

    // Update tree
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
