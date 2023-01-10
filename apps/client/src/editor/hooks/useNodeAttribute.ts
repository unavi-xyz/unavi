import { NodeJSON } from "engine";
import { useEffect, useState } from "react";

import { useEditorStore } from "../store";

export function useNodeAttribute<T extends keyof NodeJSON>(id: string | null, attribute: T) {
  const engine = useEditorStore((state) => state.engine);

  const [value, setValue] = useState<NodeJSON[T] | null>(null);

  useEffect(() => {
    if (!engine || !id) {
      setValue(null);
      return;
    }

    const newNode = engine.modules.scene.node.store.get(id) ?? null;

    if (!newNode || !attribute) {
      setValue(null);
      return;
    }

    const json = engine.modules.scene.node.toJSON(newNode);
    const initialValue = json[attribute];
    setValue(initialValue);

    function onChange(e: any) {
      if (!engine || !attribute || !newNode) return;
      if (e.attribute !== attribute) return;

      const json = engine.modules.scene.node.toJSON(newNode);
      const newValue = json[attribute];
      setValue(newValue);
    }

    newNode.addEventListener("change", onChange);

    return () => {
      newNode.removeEventListener("change", onChange);
    };
  }, [id, attribute, engine]);

  return value;
}
