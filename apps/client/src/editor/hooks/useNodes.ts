import { Node } from "@gltf-transform/core";
import { subscribe } from "engine";
import { useEffect, useState } from "react";

import { useEditor } from "../components/Editor";

export function useNodes() {
  const { engine } = useEditor();

  const [nodes, setNodes] = useState<Node[]>([]);

  useEffect(() => {
    if (!engine) return;
    const root = engine.scene.doc.getRoot();
    return subscribe(root, "Nodes", setNodes);
  }, [engine]);

  return nodes;
}
