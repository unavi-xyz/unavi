import { Engine } from "engine";

import { deepNodeChildren } from "./deepNodeChildren";

export function moveNode(nodeId: string, targetIndex: number, treeIds: string[], engine: Engine) {
  const node = engine.scene.node.store.get(nodeId);
  if (!node) throw new Error("Node not found");

  const newTreeIds = [...treeIds];
  const draggingIndex = newTreeIds.indexOf(nodeId);

  // Remove dragged nodes from tree
  const children = deepNodeChildren(node, engine);
  const draggedIds = newTreeIds.splice(draggingIndex, children.length + 1);

  // Calculate offset
  const isAfter = draggingIndex >= targetIndex;
  const offset = isAfter ? 0 : -1 * draggedIds.length;

  // Add dragged nodes to tree
  newTreeIds.splice(targetIndex + offset, 0, ...draggedIds);

  return newTreeIds;
}
