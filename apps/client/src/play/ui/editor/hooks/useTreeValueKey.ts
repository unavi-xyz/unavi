import { editorStore, TreeItem } from "@unavi/engine";
import { useAtom } from "jotai";
import { useMemo } from "react";

type ArrayKey<T extends keyof TreeItem> = Exclude<
  T,
  {
    [K in T]: TreeItem[K] extends Array<any> | Record<string, any> ? never : K;
  }[T]
>;

type Keys = ArrayKey<keyof TreeItem>;

export function useTreeValueKey<T extends Keys, U extends keyof TreeItem[T]>(
  id: bigint | undefined,
  key: T,
  index: U
): TreeItem[T][U] | undefined {
  const [items] = useAtom(editorStore.items);

  const value = useMemo(() => {
    if (!id) return undefined;
    return items.get(id)?.[key][index];
  }, [id, key, index, items]);

  return value;
}
