import { useEditorStore } from "../store";

export class RemoveMaterialAction {
  constructor(materialId: string) {
    const { engine } = useEditorStore.getState();
    if (!engine) throw new Error("Engine not found");

    // Remove material
    engine.scene.removeMaterial(materialId);

    useEditorStore.setState({ changesToSave: true });
  }
}

export function removeMaterial(materialId: string) {
  return new RemoveMaterialAction(materialId);
}
