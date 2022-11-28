import { NodeJSON } from "engine";

import { useEditorStore } from "../store";

export class UpdateNodeAction {
  constructor(nodeId: string, data: Partial<NodeJSON>) {
    const { engine } = useEditorStore.getState();
    if (!engine) throw new Error("Engine not found");

    // Update node
    engine.scene.updateNode(nodeId, data);

    useEditorStore.setState({ changesToSave: true });
  }
}

export function updateNode(nodeId: string, data: Partial<NodeJSON>) {
  return new UpdateNodeAction(nodeId, data);
}
