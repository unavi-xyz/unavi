import { MaterialJSON } from "engine";

import { useEditorStore } from "../store";

export class UpdateMaterialAction {
  constructor(materialId: string, data: Partial<MaterialJSON>) {
    const { engine } = useEditorStore.getState();
    if (!engine) throw new Error("Engine not found");

    // Update material
    engine.scene.updateMaterial(materialId, data);

    useEditorStore.setState({ changesToSave: true });
  }
}

export function updateMaterial(materialId: string, data: Partial<MaterialJSON>) {
  return new UpdateMaterialAction(materialId, data);
}
