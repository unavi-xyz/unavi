import { Node } from "@gltf-transform/core";
import { Engine } from "engine";

export function deepNodeChildren(node: Node, engine: Engine): Node[] {
  return node.listChildren().reduce((acc, child) => {
    const childChildren = deepNodeChildren(child, engine);
    return [...acc, child, ...childChildren];
  }, [] as Node[]);
}
