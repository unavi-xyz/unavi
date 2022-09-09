import { useEditorStore } from "../store";

export class RemoveEntityAction {
  constructor(entityId: string) {
    const { scene, engine } = useEditorStore.getState();

    // Remove from parent
    const entity = scene.entities[entityId];
    if (entity.parent) {
      const parent = scene.entities[entity.parent];
      parent.children = parent.children.filter((id) => id !== entityId);
      parent.children = [...parent.children];
    }

    // Remove entity
    delete scene.entities[entityId];

    // Update scene
    useEditorStore.setState({ scene });

    // Update engine
    if (engine) engine.renderThread.removeEntity(entityId);
  }
}

export function removeEntity(entityId: string) {
  return new RemoveEntityAction(entityId);
}
