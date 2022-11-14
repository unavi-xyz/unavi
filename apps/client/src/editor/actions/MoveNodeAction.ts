import { useEditorStore } from "../store";

export class MoveNodeAction {
  constructor(nodeId: string, parentId: string, index?: number) {
    const { engine, getNode } = useEditorStore.getState();
    const node = getNode(nodeId);
    if (!engine) throw new Error("Engine not found");
    if (!node) throw new Error("Node not found");

    // Set new parent
    engine.scene.updateNode(nodeId, { parentId });

    // Place node at index
    if (index !== undefined) {
      const parent = getNode(parentId);
      if (!parent) throw new Error("Parent not found");

      const sorted = parent.childrenIds.filter((id) => id !== nodeId);
      sorted.splice(index, 0, nodeId);

      parent.childrenIds$.next(sorted);
    }

    useEditorStore.setState({ changesToSave: true });
  }
}

export function moveNode(nodeId: string, parentId: string, index?: number) {
  return new MoveNodeAction(nodeId, parentId, index);
}
