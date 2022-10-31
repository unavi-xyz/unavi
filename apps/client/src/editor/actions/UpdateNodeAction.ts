import { NodeJSON } from "@wired-labs/engine";

import { useEditorStore } from "../store";

export class UpdateNodeAction {
  constructor(nodeId: string, data: Partial<NodeJSON>) {
    const { engine, selectedId } = useEditorStore.getState();
    if (!engine) throw new Error("Engine not found");

    // This is a hack to force TransformControls to detach and reattach to the new object
    if (data.meshId !== undefined && selectedId === nodeId) {
      useEditorStore.setState({ selectedId: null });
      setTimeout(() => useEditorStore.setState({ selectedId }));
    }

    // Update node
    engine.scene.updateNode(nodeId, data);
  }
}

export function updateNode(nodeId: string, data: Partial<NodeJSON>) {
  return new UpdateNodeAction(nodeId, data);
}
