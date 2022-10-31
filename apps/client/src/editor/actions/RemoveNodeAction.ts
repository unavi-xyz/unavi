import { useEditorStore } from "../store";

export class RemoveNodeAction {
  constructor(nodeId: string) {
    const { engine } = useEditorStore.getState();
    if (!engine) throw new Error("Engine not found");

    // Remove node
    engine.scene.removeNode(nodeId);
  }
}

export function removeNode(nodeId: string) {
  return new RemoveNodeAction(nodeId);
}
