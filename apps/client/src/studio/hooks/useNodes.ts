import { Node } from "@gltf-transform/core";
import { subscribe } from "@unavi/engine";
import { useEffect, useState } from "react";

import { useStudio } from "../components/Studio";

export function useNodes() {
  const { engine } = useStudio();

  const [nodes, setNodes] = useState<Node[]>([]);

  useEffect(() => {
    if (!engine) return;
    const root = engine.scene.doc.getRoot();
    return subscribe(root, "Nodes", setNodes);
  }, [engine]);

  return nodes;
}
