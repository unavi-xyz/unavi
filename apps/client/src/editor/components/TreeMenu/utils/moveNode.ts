import { useEditorStore } from "@/app/editor/[id]/store";

import { deepNodeChildren } from "./deepNodeChildren";

export function moveNode(nodeId: string, targetIndex: number) {
  const { engine, treeIds } = useEditorStore.getState();
  if (!engine) return;

  const node = engine.scene.node.store.get(nodeId);
  if (!node) throw new Error("Node not found");

  const newTreeIds = [...treeIds];
  const draggingIndex = newTreeIds.indexOf(nodeId);

  // Remove dragged nodes from tree
  const children = deepNodeChildren(node);
  const draggedIds = newTreeIds.splice(draggingIndex, children.length + 1);

  // Calculate offset
  const isAfter = draggingIndex >= targetIndex;
  const offset = isAfter ? 0 : -1 * draggedIds.length;

  // Add dragged nodes to tree
  newTreeIds.splice(targetIndex + offset, 0, ...draggedIds);
  useEditorStore.setState({ treeIds: newTreeIds });
}
