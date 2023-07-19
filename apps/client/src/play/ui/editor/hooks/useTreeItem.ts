import { useSceneStore } from "@unavi/react-client";
import { useMemo } from "react";

export function useTreeItem(id: bigint | undefined) {
  const items = useSceneStore((state) => state.items);

  const item = useMemo(() => {
    if (!id) return undefined;
    return items.get(id);
  }, [id, items]);

  return item;
}
