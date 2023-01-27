import { PrimitiveJSON } from "engine";
import { useEffect, useState } from "react";

import { useEditorStore } from "../store";

export function usePrimitiveAttribute<T extends keyof PrimitiveJSON>(
  id: string | null,
  attribute: T
) {
  const engine = useEditorStore((state) => state.engine);

  const [value, setValue] = useState<PrimitiveJSON[T] | null>(null);

  useEffect(() => {
    if (!engine || !id) {
      setValue(null);
      return;
    }

    const newPrimitive = engine.scene.primitive.store.get(id) ?? null;

    if (!newPrimitive || !attribute) {
      setValue(null);
      return;
    }

    const json = engine.scene.primitive.toJSON(newPrimitive);
    const initialValue = json[attribute];
    setValue(initialValue);

    function onChange(e: any) {
      if (!engine || !attribute || !newPrimitive) return;
      if (e.attribute !== attribute) return;

      const json = engine.scene.primitive.toJSON(newPrimitive);
      const newValue = json[attribute];
      setValue(newValue);
    }

    newPrimitive.addEventListener("change", onChange);

    return () => {
      newPrimitive.removeEventListener("change", onChange);
    };
  }, [id, attribute, engine]);

  return value;
}
