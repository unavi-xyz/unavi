import { Material } from "@wired-labs/engine";

import { useEditorStore } from "../store";

export class EditMaterialAction {
  constructor(material: Material) {
    const { scene, engine } = useEditorStore.getState();
  }
}

export function editMaterial(material: Material) {
  return new EditMaterialAction(material);
}
