import { Object3D } from "three";

import { Entity } from "../../scene";

export function copyTransform(object: Object3D, entity: Entity) {
  object.position.fromArray(entity.position);
  object.quaternion.fromArray(entity.rotation);
  object.scale.fromArray(entity.scale);
}
