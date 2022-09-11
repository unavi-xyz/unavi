import { Material } from "@wired-labs/engine";

import { useEditorStore } from "../store";
import { removeMaterial } from "./RemoveMaterialAction";

export class AddMaterialAction {
  constructor(material: Material) {
    const { scene, engine } = useEditorStore.getState();

    // Delete previous material if it exists
    if (scene.entities[material.id]) {
      removeMaterial(material.id);
    }

    // Add material
    scene.materials[material.id] = material;

    // Update scene
    useEditorStore.setState({ scene });

    // Update engine
    if (engine) engine.renderThread.addMaterial(material);
  }
}

export function addMaterial(material: Material) {
  return new AddMaterialAction(material);
}
