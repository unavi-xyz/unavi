import { editorStore } from "@unavi/engine";
import { useAtom } from "jotai";
import { useMemo } from "react";

export function useTreeItem(id: bigint | undefined) {
  const [items] = useAtom(editorStore.items);

  const item = useMemo(() => {
    if (!id) return undefined;
    return items.get(id);
  }, [id, items]);

  return item;
}
