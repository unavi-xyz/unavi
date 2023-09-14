import { NodeItem, useSceneStore } from "@unavi/engine";

export function useNodeValue<T extends keyof NodeItem>(
  entityId: bigint | undefined,
  key: T
): NodeItem[T] | undefined {
  const value = useSceneStore((state) =>
    entityId ? state.items.get(entityId)?.[key] : undefined
  );
  return value;
}
