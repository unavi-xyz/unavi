import { BufferAttribute } from "three";

import { RenderScene } from "../RenderScene";
import { SceneMap } from "./SceneMap";

export function createAttribute(
  accessorId: string,
  map: SceneMap,
  scene: RenderScene
): BufferAttribute {
  const created = map.attributes.get(accessorId);
  if (created) return created;

  const accessor = scene.accessors[accessorId];

  const attribute = new BufferAttribute(
    accessor.array,
    accessor.elementSize,
    accessor.normalized
  );

  map.attributes.set(accessorId, attribute);
  return attribute;
}
