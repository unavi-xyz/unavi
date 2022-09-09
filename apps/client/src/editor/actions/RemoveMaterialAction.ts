import { useEditorStore } from "../store";

export class RemoveMaterialAction {
  constructor(materialId: string) {
    const { scene, engine } = useEditorStore.getState();

    // Remove material
    delete scene.materials[materialId];

    // Update scene
    useEditorStore.setState({ scene });

    // Update engine
    if (engine) engine.renderThread.removeMaterial(materialId);
  }
}

export function removeMaterial(materialId: string) {
  return new RemoveMaterialAction(materialId);
}
