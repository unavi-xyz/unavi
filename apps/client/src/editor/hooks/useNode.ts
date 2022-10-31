import { Node } from "@wired-labs/engine";
import { useEffect, useState } from "react";

import { useEditorStore } from "../store";

export function useNode<T = Node>(
  id: string | null,
  callback?: (node: Node) => T
) {
  const entities$ = useEditorStore((state) => state.engine?.scene.entities$);

  const [value, setValue] = useState<T | null>(null);

  useEffect(() => {
    if (!id || !entities$) return;

    const subscription = entities$.subscribe({
      next: (entities) => {
        const node = entities[id];
        if (!node) return;

        if (callback) setValue(callback(node));
        else setValue(node as T);
      },
    });

    return () => subscription.unsubscribe();
  }, [id, callback, entities$]);

  return value;
}
