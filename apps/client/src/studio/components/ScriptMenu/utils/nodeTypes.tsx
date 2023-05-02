import { NodeTypes } from "reactflow";

import Node from "../Node";
import { getNodeSpecJSON } from "./getNodeSpecJSON";

const spec = getNodeSpecJSON();

export const nodeTypes = spec.reduce((nodes, node) => {
  nodes[node.type] = (props) => <Node spec={node} {...props} />;
  return nodes;
}, {} as NodeTypes);
