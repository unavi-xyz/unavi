import { getNodeSpecJSON } from "./utils/getNodeSpecJSON";

const allNodes = getNodeSpecJSON();

const HIDDEN_NODES = [
  "scene/get/boolean",
  "scene/get/float",
  "scene/get/string",
  "scene/get/vec2",
  "scene/set/boolean",
  "scene/set/float",
  "scene/set/string",
  "scene/set/vec2",
];

export const usedNodes = allNodes.filter((node) => {
  const type = node.type.toLowerCase();

  if (type.includes("color")) return false;
  if (type.includes("euler")) return false;
  if (type.includes("integer")) return false;
  if (type.includes("vec4")) return false;
  if (type.includes("mat3")) return false;
  if (type.includes("mat4")) return false;
  if (type.includes("customevent")) return false;

  if (HIDDEN_NODES.includes(node.type)) return false;

  return true;
});

export const categorizedNodes = usedNodes.reduce((acc, node) => {
  const category = node.type.split("/")[0];
  if (!category) return acc;

  const categoryNodes = acc[category];

  if (!categoryNodes) acc[category] = [node];
  else categoryNodes.push(node);

  return acc;
}, {} as Record<string, typeof usedNodes>);

export const categories = Object.keys(categorizedNodes);
