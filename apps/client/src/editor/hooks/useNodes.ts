import { Node } from "@gltf-transform/core";
import { useEffect, useState } from "react";

import { useEditorStore } from "../store";

export function useNodes() {
  const engine = useEditorStore((state) => state.engine);

  const [nodes, setNodes] = useState<Node[]>([]);

  useEffect(() => {
    if (!engine) return;
    const root = engine.scene.doc.getRoot();

    const initialNodes = root.listNodes();
    setNodes(initialNodes);

    function onChange(e: any) {
      if (e.attribute !== "nodes") return;
      const newNodes = root.listNodes();
      setNodes(newNodes);
    }

    root.addEventListener("change", onChange);

    return () => {
      root.removeEventListener("change", onChange);
    };
  }, [engine]);

  return nodes;
}
