import { SceneMap } from "../types";
import { getChildren } from "../utils/getChildren";
import { removeNodeObject } from "./removeNodeObject";

export function removeNode(nodeId: string, map: SceneMap) {
  // Repeat for children
  const children = getChildren(nodeId, map);
  children.forEach((child) => removeNode(child.id, map));

  // Remove node
  map.nodes.delete(nodeId);

  // Remove object
  removeNodeObject(nodeId, map);
}
