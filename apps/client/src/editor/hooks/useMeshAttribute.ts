import { MeshJSON } from "engine";
import { useEffect, useState } from "react";

import { useEditorStore } from "../store";

export function useMeshAttribute<T extends keyof MeshJSON>(id: string | null, attribute: T) {
  const engine = useEditorStore((state) => state.engine);

  const [value, setValue] = useState<MeshJSON[T] | null>(null);

  useEffect(() => {
    if (!engine || !id) {
      setValue(null);
      return;
    }

    const newMesh = engine.scene.mesh.store.get(id) ?? null;

    if (!newMesh || !attribute) {
      setValue(null);
      return;
    }

    const json = engine.scene.mesh.toJSON(newMesh);
    const initialValue = json[attribute];
    setValue(initialValue);

    function onChange(e: any) {
      if (!engine || !attribute || !newMesh) return;
      if (e.attribute !== attribute) return;

      const json = engine.scene.mesh.toJSON(newMesh);
      const newValue = json[attribute];
      setValue(newValue);
    }

    newMesh.addEventListener("change", onChange);

    return () => {
      newMesh.removeEventListener("change", onChange);
    };
  }, [id, attribute, engine]);

  return value;
}
