import { Entity } from "@wired-labs/engine";

import { useEditorStore } from "../store";
import { removeEntity } from "./RemoveEntityAction";

export class AddEntityAction {
  constructor(entity: Entity) {
    const { scene, engine } = useEditorStore.getState();

    // Delete previous entity if it exists
    if (scene.entities[entity.id]) {
      removeEntity(entity.id);
    }

    // Add entity
    scene.entities[entity.id] = entity;

    // Add to parent if not already there
    if (entity.parent) {
      const parent = scene.entities[entity.parent];
      if (!parent.children.includes(entity.id)) {
        parent.children.push(entity.id);
        parent.children = [...parent.children];
      }
    }

    // Update scene
    useEditorStore.setState({ scene });

    // Update engine
    if (engine) engine.renderThread.addEntity(entity);
  }
}

export function addEntity(entity: Entity) {
  return new AddEntityAction(entity);
}
