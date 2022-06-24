import { IEntity, Transform } from "./types";

export function findEntityById(
  entity: IEntity,
  id: string
): IEntity | undefined {
  if (entity.id === id) return entity;

  for (const child of entity.children) {
    const found = findEntityById(child, id);
    if (found) return found;
  }

  return undefined;
}

export function traverseTree(
  entity: IEntity,
  callback: (entity: IEntity) => void
) {
  callback(entity);

  for (const child of entity.children) {
    traverseTree(child, callback);
  }
}

export function areTransformsEqual(a: Transform, b: Transform) {
  const positionDiff = a.position.find((value, index) => {
    if (value !== b.position[index]) {
      return true;
    }
  });
  if (positionDiff) return false;

  const rotationDiff = a.rotation.find((value, index) => {
    if (value !== b.rotation[index]) {
      return true;
    }
  });
  if (rotationDiff) return false;

  const scaleDiff = a.scale.find((value, index) => {
    if (value !== b.scale[index]) {
      return true;
    }
  });
  if (scaleDiff) return false;

  return true;
}
