import { Mesh } from "@gltf-transform/core";
import { useEffect, useState } from "react";

import { useStudio } from "../components/Studio";

export function useMesh(id: string | null) {
  const { engine } = useStudio();

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
