import { useEditorStore } from "../store";

export class RemoveMeshAction {
  constructor(meshId: string) {
    const { engine } = useEditorStore.getState();
    if (!engine) throw new Error("Engine not found");

    // Remove mesh
    engine.scene.removeMesh(meshId);

    useEditorStore.setState({ changesToSave: true });
  }
}

export function removeMesh(meshId: string) {
  return new RemoveMeshAction(meshId);
}
