import { Entity } from "@wired-labs/engine";

import { useEditorStore } from "../store";

export class SetTransform {
  constructor(entity: Entity) {
    const { tree, engine } = useEditorStore.getState();

    // Set entity
    tree[entity.id] = entity;

    // Update tree
    useEditorStore.setState({ tree });

    // Update engine
    if (engine) engine.renderThread.setTransform(entity);
  }
}

export function setTransform(entity: Entity) {
  return new SetTransform(entity);
}
