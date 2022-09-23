import { Entity } from "@wired-labs/engine";

import { useEditorStore } from "../store";

export class AddEntityAction {
  constructor(entity: Entity) {
    const { engine } = useEditorStore.getState();
    if (!engine) throw new Error("Engine not found");

    // Add entity
    engine.scene.addEntity(entity);
  }
}

export function addEntity(entity: Entity) {
  return new AddEntityAction(entity);
}
