import { Entity } from "@wired-labs/engine";

import { useStudioStore } from "../store";

export class SetEntity {
  constructor(entity: Entity) {
    const { tree, engine } = useStudioStore.getState();

    // Set entity
    tree[entity.id] = entity;

    // Update tree
    useStudioStore.setState({ tree });

    // Update engine
    if (engine) engine.renderThread.setEntity(entity);
  }
}

export function setEntity(entity: Entity) {
  return new SetEntity(entity);
}
