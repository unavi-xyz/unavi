import { Entity } from "@wired-labs/engine";

import { useStudioStore } from "../store";

export class SetTransform {
  constructor(entity: Entity) {
    const { tree, engine } = useStudioStore.getState();

    // Set entity
    tree[entity.id] = entity;

    // Update tree
    useStudioStore.setState({ tree });

    // Update engine
    if (engine) engine.renderThread.setTransform(entity);
  }
}

export function setTransform(entity: Entity) {
  return new SetTransform(entity);
}
