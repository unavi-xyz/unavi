import { NodeJSON } from "../../../scene";
import { SceneMap } from "../types";

export function getChildren(nodeId: string, map: SceneMap) {
  const children: NodeJSON[] = [];

  map.nodes.forEach((e) => {
    if (e.parentId === nodeId) children.push(e);
  });

  return children;
}
