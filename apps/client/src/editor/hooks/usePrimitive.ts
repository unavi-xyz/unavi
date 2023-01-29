import { Primitive } from "@gltf-transform/core";
import { useEffect, useState } from "react";

import { useEditorStore } from "../store";

export function usePrimitive(id: string | null) {
  const engine = useEditorStore((state) => state.engine);

  const [primitive, setPrimitive] = useState<Primitive | null>(null);

  useEffect(() => {
    if (!engine || !id) {
      setPrimitive(null);
      return;
    }

    const newPrimitive = engine.scene.primitive.store.get(id) ?? null;
    setPrimitive(newPrimitive);
  }, [id, engine]);

  return primitive;
}
