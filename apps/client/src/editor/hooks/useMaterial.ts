import { Material } from "engine";
import { useEffect, useState } from "react";

import { useEditorStore } from "../store";

export function useMaterial<T = Material>(id: string | null, callback?: (material: Material) => T) {
  const materials$ = useEditorStore((state) => state.engine?.scene.materials$);

  const [value, setValue] = useState<T | null>(null);

  useEffect(() => {
    if (!id || !materials$) return;

    const subscription = materials$.subscribe({
      next: (materials) => {
        const material = materials[id];
        if (!material) return;

        if (callback) {
          setValue(callback(material));
        } else {
          setValue(material as T);
        }
      },
    });

    return () => subscription.unsubscribe();
  }, [id, callback, materials$]);

  return value;
}
