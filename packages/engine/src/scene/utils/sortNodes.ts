import { MeshJSON } from "../mesh/types";
import { NodeJSON, SceneJSON } from "../types";

/*
 * Sort nodes into a loadable order.
 * Sorts by depth, so that children are added after their parents.
 * Put all skins last, so that they are added after their bones.
 */
export function sortNodes(scene: Partial<SceneJSON>): NodeJSON[] {
  const nodes = scene.nodes || [];
  const meshes = scene.meshes || [];

  return nodes.sort((a, b) => {
    const aDepth = nodeDepth(nodes, a);
    const bDepth = nodeDepth(nodes, b);

    const aSkin = hasSkin(a, meshes);
    const bSkin = hasSkin(b, meshes);

    if (aSkin && bSkin) return 0;
    if (aSkin) return 1;
    if (bSkin) return -1;

    return aDepth - bDepth;
  });
}

function hasSkin(node: NodeJSON, meshes: MeshJSON[]) {
  const mesh = meshes.find((m) => m.id === node.meshId);
  return mesh?.type === "Primitives" && mesh.primitives.some((p) => p.skin);
}

function nodeDepth(nodes: NodeJSON[], node: NodeJSON): number {
  if (node.parentId === "root") return 0;

  const parent = nodes.find((e) => e.id === node.parentId);
  if (!parent) return 0;

  return nodeDepth(nodes, parent) + 1;
}
