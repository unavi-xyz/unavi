import { useSceneStore } from "@unavi/engine";
import { useMemo } from "react";

export function useTreeItem(entityId: bigint | undefined) {
  const items = useSceneStore((state) => state.items);

  const item = useMemo(() => {
    if (!entityId) return undefined;
    const item = items.get(entityId);
    return item;
  }, [entityId, items]);

  return item;
}
