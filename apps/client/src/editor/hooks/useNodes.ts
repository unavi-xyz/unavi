import { Node } from "@gltf-transform/core";
import { subscribe } from "engine";
import { useEffect, useState } from "react";

import { useEditorStore } from "@/app/editor/[id]/store";

export function useNodes() {
  const engine = useEditorStore((state) => state.engine);

  const [nodes, setNodes] = useState<Node[]>([]);

  useEffect(() => {
    if (!engine) return;
    const root = engine.scene.doc.getRoot();
    return subscribe(root, "Nodes", setNodes);
  }, [engine]);

  return nodes;
}
