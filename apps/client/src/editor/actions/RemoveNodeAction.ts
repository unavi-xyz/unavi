import { useEditorStore } from "../store";

export class RemoveNodeAction {
  constructor(nodeId: string) {
    const { engine } = useEditorStore.getState();
    if (!engine) throw new Error("Engine not found");

    const node = engine.scene.nodes[nodeId];
    if (!node) throw new Error("Node not found");

    // Remove node
    engine.scene.removeNode(nodeId);

    useEditorStore.setState({ changesToSave: true });
  }
}

export function removeNode(nodeId: string) {
  return new RemoveNodeAction(nodeId);
}
