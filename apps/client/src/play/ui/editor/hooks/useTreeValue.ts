import { editorStore, TreeItem } from "@unavi/engine";
import { useAtom } from "jotai";
import { useMemo } from "react";

export function useTreeValue<T extends keyof TreeItem>(
  id: bigint | undefined,
  key: T
): TreeItem[T] | undefined {
  const [items] = useAtom(editorStore.items);

  const value = useMemo(() => {
    if (!id) return undefined;
    return items.get(id)?.[key];
  }, [id, key, items]);

  return value;
}
