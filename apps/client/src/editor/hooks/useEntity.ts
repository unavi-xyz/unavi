import { Entity } from "@wired-labs/engine";
import { useEffect, useState } from "react";

import { useEditorStore } from "../store";

export function useEntity<T = Entity>(
  id: string | null,
  callback?: (entity: Entity) => T
) {
  const entities$ = useEditorStore((state) => state.engine?.scene.entities$);

  const [value, setValue] = useState<T | null>(null);

  useEffect(() => {
    if (!id || !entities$) return;

    const subscription = entities$.subscribe({
      next: (entities) => {
        const entity = entities[id];
        if (!entity) return;

        if (callback) {
          setValue(callback(entity));
        } else {
          setValue(entity as T);
        }
      },
    });

    return () => subscription.unsubscribe();
  }, [id, callback, entities$]);

  return value;
}
