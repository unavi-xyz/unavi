import { Engine } from "engine";

/**
 * Checks if the target node is an ancestor of another node
 * @param targetId The id of the potential ancestor node
 * @param nodeId The id of the node to start the search from
 * @param engine
 */
export function isAncestor(targetId: string, nodeId: string, engine: Engine): boolean {
  if (!engine) return false;

  if (nodeId === targetId) return true;

  const node = engine.modules.scene.node.store.get(nodeId);
  if (!node) throw new Error("Node not found");

  const parentId = engine.modules.scene.node.getParent(node);
  if (!parentId) return false;

  return isAncestor(targetId, parentId, engine);
}
