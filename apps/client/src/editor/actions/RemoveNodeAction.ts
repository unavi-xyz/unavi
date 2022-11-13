import { useEditorStore } from "../store";
import { removeMesh } from "./RemoveMeshAction";

export class RemoveNodeAction {
  constructor(nodeId: string) {
    const { engine } = useEditorStore.getState();
    if (!engine) throw new Error("Engine not found");

    const node = engine.scene.nodes[nodeId];
    if (!node) throw new Error("Node not found");

    // Remove node
    engine.scene.removeNode(nodeId);

    // Remove mesh
    if (node.meshId) removeMesh(node.meshId);

    useEditorStore.setState({ changesToSave: true });
  }
}

export function removeNode(nodeId: string) {
  return new RemoveNodeAction(nodeId);
}
