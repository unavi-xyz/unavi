import { Node } from "engine";
import { useEffect, useState } from "react";

import { useEditorStore } from "../store";

export function useNode<T = Node>(id: string | null, callback?: (node: Node) => T) {
  const nodes$ = useEditorStore((state) => state.engine?.scene.nodes$);

  const [value, setValue] = useState<T | null>(null);

  useEffect(() => {
    if (!id || !nodes$) return;

    const subscription = nodes$.subscribe({
      next: (nodes) => {
        const node = nodes[id];
        if (!node) return;

        if (callback) setValue(callback(node));
        else setValue(node as T);
      },
    });

    return () => subscription.unsubscribe();
  }, [id, callback, nodes$]);

  return value;
}
