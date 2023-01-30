import { Node } from "@gltf-transform/core";

import { useEditorStore } from "../../../store";

export function deepNodeChildren(node: Node): Node[] {
  const { engine } = useEditorStore.getState();
  if (!engine) return [];

  return node.listChildren().reduce((acc, child) => {
    const childChildren = deepNodeChildren(child);
    return [...acc, child, ...childChildren];
  }, [] as Node[]);
}
