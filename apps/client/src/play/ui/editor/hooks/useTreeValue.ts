import { TreeItem, useSceneStore } from "@unavi/engine";

export function useTreeValue<T extends keyof TreeItem>(
  entityId: bigint | undefined,
  key: T
): TreeItem[T] | undefined {
  const value = useSceneStore((state) =>
    entityId ? state.items.get(entityId)?.[key] : undefined
  );
  return value;
}
