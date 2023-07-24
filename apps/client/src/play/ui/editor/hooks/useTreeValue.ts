import { TreeItem, useSceneStore } from "@unavi/engine";

export function useTreeValue<T extends keyof TreeItem>(
  id: bigint | undefined,
  key: T
): TreeItem[T] | undefined {
  const value = useSceneStore((state) =>
    id ? state.items.get(id)?.[key] : undefined
  );

  return value;
}
