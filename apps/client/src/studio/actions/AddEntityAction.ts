import { Entity } from "@wired-labs/engine";

import { useStudioStore } from "../store";
import { removeEntity } from "./RemoveEntityAction";

export class AddEntityAction {
  constructor(entity: Entity) {
    const { tree, engine } = useStudioStore.getState();

    // Delete previous entity if it exists
    if (tree[entity.id]) {
      removeEntity(entity.id);
    }

    // Add entity
    tree[entity.id] = entity;

    // Add to parent if not already there
    if (entity.parent) {
      const parent = tree[entity.parent];
      if (!parent.children.includes(entity.id)) {
        parent.children.push(entity.id);
        parent.children = [...parent.children];
      }
    }

    // Update tree
    useStudioStore.setState({ tree });

    // Update engine
    if (engine) engine.renderThread.addEntity(entity);
  }
}

export function addEntity(entity: Entity) {
  return new AddEntityAction(entity);
}
