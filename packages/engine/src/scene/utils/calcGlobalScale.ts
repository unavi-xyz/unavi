import { Triplet } from "../../types";
import { NodeJSON } from "../types";

export function calcGlobalScale(node: NodeJSON, nodes: NodeJSON[]): Triplet {
  const parent = nodes.find((n) => n.id === node.parentId);
  if (!parent || node.parentId === "root") return node.scale;

  const parentScale = calcGlobalScale(parent, nodes);

  const newScale: Triplet = [
    node.scale[0] * parentScale[0],
    node.scale[1] * parentScale[1],
    node.scale[2] * parentScale[2],
  ];

  return newScale;
}
