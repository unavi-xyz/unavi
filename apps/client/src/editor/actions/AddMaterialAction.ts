import { Material } from "@wired-labs/engine";

import { useEditorStore } from "../store";

export class AddMaterialAction {
  constructor(material: Material) {
    const { engine } = useEditorStore.getState();
    if (!engine) throw new Error("Engine not found");

    // Add material
    engine.scene.addMaterial(material);
  }
}

export function addMaterial(material: Material) {
  return new AddMaterialAction(material);
}
