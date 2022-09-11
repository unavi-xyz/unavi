import { Entity } from "@wired-labs/engine";

import { useEditorStore } from "../store";

export class SetTransform {
  constructor(entity: Entity) {
    const { scene, engine } = useEditorStore.getState();

    // Set entity
    scene.entities[entity.id] = entity;

    // Update scene
    useEditorStore.setState({ scene });

    // Update engine
    if (engine) engine.renderThread.setTransform(entity);
  }
}

export function setTransform(entity: Entity) {
  return new SetTransform(entity);
}
