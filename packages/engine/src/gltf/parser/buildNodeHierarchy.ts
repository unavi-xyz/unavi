import { Object3D } from "three";

import { GLTF } from "../schemaTypes";

export async function buildNodeHierarchy(
  index: number,
  json: GLTF,
  loadNode: (index: number) => Promise<Object3D>,
  _buildNodeHierarchy: (index: number) => Promise<Object3D>
): Promise<Object3D> {
  const node = await loadNode(index);

  if (json.nodes === undefined) {
    throw new Error("No nodes found");
  }

  const nodeDef = json.nodes[index];

  // Children
  if (nodeDef.children) {
    const childrenPromises = nodeDef.children.map((childIndex) =>
      _buildNodeHierarchy(childIndex)
    );
    const children = await Promise.all(childrenPromises);
    node.add(...children);
  }

  return node;
}
