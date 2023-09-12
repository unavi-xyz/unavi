import { useSceneStore } from "@unavi/engine";
import { useMemo } from "react";

export function useTreeItem(id: bigint | undefined) {
  const items = useSceneStore((state) => state.items);

  const item = useMemo(() => {
    if (!id) return undefined;
    const item = items.get(id);
    return item;
  }, [id, items]);

  return item;
}
