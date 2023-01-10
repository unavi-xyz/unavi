import { Object3D } from "three";

import { NodeJSON } from "../../../scene";

export function copyTransform(object: Object3D, node: NodeJSON) {
  object.position.fromArray(node.position);
  object.quaternion.fromArray(node.rotation);
  object.scale.fromArray(node.scale);
}
