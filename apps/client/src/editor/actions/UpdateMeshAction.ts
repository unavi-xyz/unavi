import { MeshJSON } from "@wired-labs/engine";

import { useEditorStore } from "../store";

export class UpdateMeshAction {
  constructor(meshId: string, data: Partial<MeshJSON>) {
    const { engine } = useEditorStore.getState();
    if (!engine) throw new Error("Engine not found");

    // Update node
    engine.scene.updateMesh(meshId, data);

    useEditorStore.setState({ changesToSave: true });
  }
}

export function updateMesh(meshId: string, data: Partial<MeshJSON>) {
  return new UpdateMeshAction(meshId, data);
}
