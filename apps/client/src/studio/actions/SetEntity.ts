import { Entity } from "@wired-labs/engine";

import { useStudioStore } from "../store";
import { updateTree } from "../utils/tree";

export class SetEntity {
  constructor(entity: Entity) {
    const { tree, engine } = useStudioStore.getState();

    // Set entity
    tree[entity.id] = entity;

    // Update tree
    useStudioStore.setState({ tree });

    // Update UI
    updateTree();

    // Update engine
    if (engine) engine.renderThread.setEntity(entity);
  }
}

export function setEntity(entity: Entity) {
  return new SetEntity(entity);
}
