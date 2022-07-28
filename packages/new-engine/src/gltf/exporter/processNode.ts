import { Line, Mesh, Object3D, Points, SkinnedMesh } from "three";

import { arraysEqual } from "../../utils/arraysEqual";
import { GLTF, Node } from "../schemaTypes";

export function processNode(
  node: Object3D,
  json: GLTF,
  skins: SkinnedMesh[],
  processMesh: (node: Mesh | Line | Points) => number | null,
  _processNode: (node: Object3D) => number
) {
  const nodeDef: Node = {};
  if (node.name) nodeDef.name = node.name;

  // Translation, rotation, scale
  const position = node.position.toArray();
  const rotation = node.quaternion.toArray();
  const scale = node.scale.toArray();

  if (!arraysEqual(position, [0, 0, 0])) nodeDef.translation = position;
  if (!arraysEqual(rotation, [0, 0, 0, 1])) nodeDef.rotation = rotation;
  if (!arraysEqual(scale, [1, 1, 1])) nodeDef.scale = scale;

  // Mesh
  if (node instanceof Mesh || node instanceof Line || node instanceof Points) {
    const meshIndex = processMesh(node);
    if (meshIndex !== null) nodeDef.mesh = meshIndex;
  }

  // Mark if skinned mesh
  if (node instanceof SkinnedMesh) skins.push(node);

  // Children
  if (node.children.length > 0) {
    const children: number[] = [];

    node.children.forEach((child) => {
      if (!child.visible) return;

      const childIndex = _processNode(child);
      children.push(childIndex);
    });

    if (children.length > 0) nodeDef.children = children;
  }

  // Add to json
  if (!json.nodes) json.nodes = [];
  const index = json.nodes.push(nodeDef) - 1;
  return index;
}
