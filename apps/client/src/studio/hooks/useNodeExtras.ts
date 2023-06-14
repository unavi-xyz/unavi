import { Node } from "@gltf-transform/core";
import { NodeExtras } from "@unavi/engine";

import { useSubscribe } from "./useSubscribe";

export function useNodeExtras(node: Node | null) {
  return useSubscribe(node, "Extras") as NodeExtras | undefined;
}
