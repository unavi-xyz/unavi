import { Node } from "@gltf-transform/core";
import { useEffect, useState } from "react";

import { useEditorStore } from "../../../app/editor/[id]/store";

export function useNode(id: string | null) {
  const engine = useEditorStore((state) => state.engine);

  const [node, setNode] = useState<Node | null>(null);

  useEffect(() => {
    if (!engine || !id) {
      setNode(null);
      return;
    }

    const newNode = engine.scene.node.store.get(id) ?? null;
    setNode(newNode);
  }, [id, engine]);

  return node;
}
