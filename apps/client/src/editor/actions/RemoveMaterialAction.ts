import { useEditorStore } from "../store";
import { setMaterial } from "./SetMaterial";

export class RemoveMaterialAction {
  constructor(materialId: string) {
    const { scene, engine } = useEditorStore.getState();

    // Remove from all entities
    let didRemove = false;
    Object.values(scene.entities).forEach((entity) => {
      switch (entity.type) {
        case "Box":
        case "Sphere":
        case "Cylinder":
          if (entity.material === materialId) {
            setMaterial(entity.id, undefined);
            didRemove = true;
          }
      }
    });

    // Remove material
    delete scene.materials[materialId];

    // Update scene
    useEditorStore.setState({ scene });

    // Update engine
    if (engine && didRemove) engine.renderThread.removeMaterial(materialId);
  }
}

export function removeMaterial(materialId: string) {
  return new RemoveMaterialAction(materialId);
}
