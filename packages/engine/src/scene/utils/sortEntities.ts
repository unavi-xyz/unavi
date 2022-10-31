import { NodeJSON } from "../types";

/*
 * Sort entities into a loadable order.
 * Sorts by depth, so that children are added after their parents.
 * Put all skins last, so that they are added after their bones.
 */
export function sortEntities<T extends NodeJSON>(entities: T[]) {
  return entities.sort((a, b) => {
    const aDepth = entityDepth(entities, a);
    const bDepth = entityDepth(entities, b);

    const aSkin = isSkin(a);
    const bSkin = isSkin(b);

    if (aSkin && bSkin) return 0;
    if (aSkin) return 1;
    if (bSkin) return -1;

    return aDepth - bDepth;
  });
}

function isSkin(e: NodeJSON) {
  return e.mesh?.type === "Primitive" && e.mesh.skin !== null;
}

function entityDepth<T extends NodeJSON>(entities: T[], node: T): number {
  if (node.parentId === "root") return 0;
  const parent = entities.find((e) => e.id === node.parentId);
  if (!parent) return 0;
  return entityDepth(entities, parent) + 1;
}
