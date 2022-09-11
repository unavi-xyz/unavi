import { useEditorStore } from "../store";

export class SetMaterial {
  constructor(entityId: string, materialId: string | undefined) {
    const { scene, engine } = useEditorStore.getState();
    const entity = scene.entities[entityId];

    // Set material
    switch (entity.type) {
      case "Box":
      case "Sphere":
      case "Cylinder":
        entity.material = materialId;
        break;
      default:
        throw new Error(
          `Entity type ${entity.type} does not support materials`
        );
    }

    // Update scene
    scene.entities[entityId] = entity;
    useEditorStore.setState({ scene });

    // Update engine
    if (engine) engine.renderThread.setMaterial(entityId, materialId ?? null);
  }
}

export function setMaterial(entityId: string, materialId: string | undefined) {
  return new SetMaterial(entityId, materialId);
}
