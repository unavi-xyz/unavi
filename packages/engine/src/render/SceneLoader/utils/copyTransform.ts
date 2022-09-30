import { Object3D } from "three";

import { EntityJSON } from "../../../scene";

export function copyTransform(object: Object3D, entity: EntityJSON) {
  object.position.fromArray(entity.position);
  object.quaternion.fromArray(entity.rotation);
  object.scale.fromArray(entity.scale);
}
