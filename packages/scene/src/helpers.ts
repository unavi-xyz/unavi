import { TreeObject } from "./types";

export function findObjectById(
  object: TreeObject,
  id: string
): TreeObject | undefined {
  if (object.id === id) return object;

  for (const child of object.children) {
    const found = findObjectById(child, id);
    if (found) return found;
  }

  return undefined;
}
