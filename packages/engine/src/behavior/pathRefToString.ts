import { Node } from "@gltf-transform/core";

import { Engine } from "../Engine";
import { ParamJsonPath } from "../gltf";

export function pathRefToString({ property }: ParamJsonPath, jsonNode: Node, engine: Engine) {
  const index = engine.scene.doc.getRoot().listNodes().indexOf(jsonNode);
  if (index === -1) return;

  return `/nodes/${index}/${property}`;
}
