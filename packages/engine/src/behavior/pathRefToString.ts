import { Engine } from "../Engine";
import { JsonPathRef } from "../gltf/extensions/Behavior/types";

export function pathRefToString(value: JsonPathRef, engine: Engine) {
  const { node, property } = value;

  const index = engine.scene.doc.getRoot().listNodes().indexOf(node);
  if (index === -1) return;

  return `/nodes/${index}/${property}`;
}
