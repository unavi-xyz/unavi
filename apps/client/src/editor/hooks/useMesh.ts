import { Mesh } from "@gltf-transform/core";
import { useEffect, useState } from "react";

import { useEditorStore } from "../store";

export function useMesh(id: string | null) {
  const engine = useEditorStore((state) => state.engine);

  const [mesh, setMesh] = useState<Mesh | null>(null);

  useEffect(() => {
    if (!engine || !id) {
      setMesh(null);
      return;
    }

    const newMesh = engine.scene.mesh.store.get(id) ?? null;
    setMesh(newMesh);
  }, [id, engine]);

  return mesh;
}
