import { useEditorStore } from "../store";

export class RemoveMaterialAction {
  constructor(materialId: string) {
    const { engine } = useEditorStore.getState();
    if (!engine) throw new Error("Engine not found");

    // Remove material
    engine.scene.removeMaterial(materialId);
  }
}

export function removeMaterial(materialId: string) {
  return new RemoveMaterialAction(materialId);
}
