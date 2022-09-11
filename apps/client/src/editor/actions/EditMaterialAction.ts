import { Material } from "@wired-labs/engine";

import { useEditorStore } from "../store";

export class EditMaterialAction {
  constructor(material: Material) {
    const { scene, engine } = useEditorStore.getState();

    // Update material
    scene.materials[material.id] = material;

    // Update scene
    useEditorStore.setState({ scene });

    // Update engine
    if (engine) engine.renderThread.editMaterial(material);
  }
}

export function editMaterial(material: Material) {
  return new EditMaterialAction(material);
}
