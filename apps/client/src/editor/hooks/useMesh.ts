import { Mesh } from "@wired-labs/engine";
import { useEffect, useState } from "react";

import { useEditorStore } from "../store";

export function useMesh<T = Mesh>(
  id: string | null,
  callback?: (mesh: Mesh) => T
) {
  const meshes$ = useEditorStore((state) => state.engine?.scene.meshes$);

  const [value, setValue] = useState<T | null>(null);

  useEffect(() => {
    if (!meshes$ || !id) {
      setValue(null);
      return;
    }

    const subscription = meshes$.subscribe({
      next: (meshes) => {
        const mesh = meshes[id];
        if (!mesh) return;

        if (callback) setValue(callback(mesh));
        else setValue(mesh as T);
      },
    });

    return () => subscription.unsubscribe();
  }, [id, callback, meshes$]);

  return value;
}
