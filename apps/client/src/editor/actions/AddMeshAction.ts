import { Mesh } from "engine";

import { useEditorStore } from "../store";

export class AddMeshAction {
  constructor(mesh: Mesh) {
    const { engine } = useEditorStore.getState();
    if (!engine) throw new Error("Engine not found");

    // Add mesh
    engine.scene.addMesh(mesh);

    useEditorStore.setState({ changesToSave: true });
  }
}

export function addMesh(mesh: Mesh) {
  return new AddMeshAction(mesh);
}
